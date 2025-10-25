# 7z Archive Streaming Optimization Analysis

## Current Situation

7z archives currently use the memory-based approach (same as before optimization) due to API constraints in the `sevenz-rust` crate.

### Why 7z Wasn't Optimized

The `sevenz-rust` crate's `SevenZReader` has the following signature:

```rust
pub struct SevenZReader<R: Read + Seek> {
    // Internal fields require &mut self for iteration
}

impl<R: Read + Seek> SevenZReader<R> {
    pub fn new(source: R, size: u64, password: Password) -> Result<Self>;

    // ❌ PROBLEM: Requires &mut self
    pub fn for_each_entries<F>(&mut self, mut func: F) -> Result<bool>
    where
        F: FnMut(&SevenZArchiveEntry, &mut dyn Read) -> Result<bool>;
}
```

Our `Archive` trait requires immutable `&self`:

```rust
pub trait Archive {
    fn find_first_image(&self, sort: bool) -> Result<ArchiveEntry>;  // ← &self
    fn extract_entry(&self, entry: &ArchiveEntry) -> Result<Vec<u8>>;  // ← &self
}
```

**Conflict**: `SevenZReader::for_each_entries()` needs `&mut self`, but we have `&self`.

---

## Solution Options

### Option 1: Use RefCell<SevenZReader> ⭐ RECOMMENDED

**Idea**: Wrap the reader in `RefCell` for interior mutability.

```rust
pub struct SevenZipArchiveFromStream<R: Read + Seek> {
    reader: RefCell<SevenZReader<R>>,
    size: u64,
}

impl<R: Read + Seek> Archive for SevenZipArchiveFromStream<R> {
    fn find_first_image(&self, sort: bool) -> Result<ArchiveEntry> {
        let mut reader = self.reader.borrow_mut();  // ✅ Get mutable access

        let mut first_image: Option<ArchiveEntry> = None;

        reader.for_each_entries(|entry, _data| {
            if is_image_file(entry.name()) {
                first_image = Some(ArchiveEntry {
                    name: entry.name().to_string(),
                    size: entry.size(),
                    is_directory: entry.is_directory(),
                });
                Ok(false)  // Stop iteration
            } else {
                Ok(true)  // Continue
            }
        })?;

        first_image.ok_or(CbxError::Archive("No images found".to_string()))
    }
}
```

**Pros**:
- ✅ Minimal code changes
- ✅ Same API as ZIP/RAR handlers
- ✅ No architectural changes needed
- ✅ Can implement immediately

**Cons**:
- ⚠️ Runtime borrow checking (could panic if used incorrectly)
- ⚠️ Small performance overhead (negligible)

**Implementation Effort**: Low (1-2 hours)
**Risk**: Low

---

### Option 2: Change Archive Trait to Use &mut self

**Idea**: Make the trait use `&mut self` instead of `&self`.

```rust
pub trait Archive {
    fn find_first_image(&mut self, sort: bool) -> Result<ArchiveEntry>;  // ← &mut
    fn extract_entry(&mut self, entry: &ArchiveEntry) -> Result<Vec<u8>>;  // ← &mut
}
```

**Pros**:
- ✅ More idiomatic Rust
- ✅ No RefCell overhead
- ✅ Compile-time safety

**Cons**:
- ❌ Breaks ALL existing implementations (ZIP, RAR, 7z)
- ❌ Need to refactor all archive handlers
- ❌ May conflict with trait object usage (`Box<dyn Archive>`)
- ❌ Large refactoring effort

**Implementation Effort**: High (1-2 days)
**Risk**: High (breaking change)

**Verdict**: Not recommended - too invasive

---

### Option 3: Separate Trait for Streaming Archives

**Idea**: Create a new `StreamingArchive` trait for archives that support streaming.

```rust
pub trait StreamingArchive {
    fn find_first_image_streaming(&mut self, sort: bool) -> Result<ArchiveEntry>;
    fn extract_entry_streaming(&mut self, entry: &ArchiveEntry) -> Result<Vec<u8>>;
}

impl StreamingArchive for SevenZipArchiveFromStream<R> {
    // Implementation with &mut self
}

// Adapter to convert StreamingArchive → Archive
impl Archive for StreamingWrapper<T: StreamingArchive> {
    fn find_first_image(&self, sort: bool) -> Result<ArchiveEntry> {
        self.inner.borrow_mut().find_first_image_streaming(sort)
    }
}
```

**Pros**:
- ✅ Clear separation of concerns
- ✅ Doesn't break existing code
- ✅ Flexible for future additions

**Cons**:
- ⚠️ More complex architecture
- ⚠️ Still needs RefCell internally
- ⚠️ More code to maintain

**Implementation Effort**: Medium (4-6 hours)
**Risk**: Medium

**Verdict**: Over-engineered for this use case

---

### Option 4: Create Reader on Each Call

**Idea**: Don't store the reader, recreate it from IStream each time.

```rust
pub struct SevenZipArchiveFromStream {
    stream: IStream,  // Keep original IStream
    size: u64,
}

impl Archive for SevenZipArchiveFromStream {
    fn find_first_image(&self, sort: bool) -> Result<ArchiveEntry> {
        // Create new reader each time
        let reader = IStreamReader::new(self.stream.clone());
        let mut sz_reader = SevenZReader::new(reader, self.size, Password::empty())?;

        // Use the reader
        sz_reader.for_each_entries(...)?;
    }
}
```

**Pros**:
- ✅ No RefCell needed
- ✅ Simple implementation
- ✅ Works with immutable &self

**Cons**:
- ❌ **Performance**: Recreates reader on each call
- ❌ May re-parse 7z headers multiple times
- ❌ IStream::clone() may be expensive

**Implementation Effort**: Low (1 hour)
**Risk**: Low

**Verdict**: Simple but inefficient - not recommended for production

---

## Recommended Implementation: RefCell Approach

### Step-by-Step Implementation

#### 1. Modify SevenZipArchiveFromStream

```rust
use std::cell::RefCell;

pub struct SevenZipArchiveFromStream<R: Read + Seek> {
    reader: RefCell<Option<SevenZReader<R>>>,  // Wrap in RefCell + Option
    source: R,  // Keep original source for recreation if needed
    size: u64,
}

impl<R: Read + Seek> SevenZipArchiveFromStream<R> {
    pub fn new(mut source: R) -> Result<Self> {
        // Get size
        let size = source.seek(SeekFrom::End(0))?;
        source.seek(SeekFrom::Start(0))?;

        // Create initial reader
        let reader = SevenZReader::new(&mut source, size, Password::empty())?;

        Ok(Self {
            reader: RefCell::new(Some(reader)),
            source,
            size,
        })
    }

    /// Get or recreate the reader
    fn get_reader(&self) -> std::cell::RefMut<SevenZReader<R>> {
        // This is a simplified version - actual implementation needs proper handling
        self.reader.borrow_mut()
    }
}
```

#### 2. Implement find_first_image

```rust
impl<R: Read + Seek> Archive for SevenZipArchiveFromStream<R> {
    fn find_first_image(&self, sort: bool) -> Result<ArchiveEntry> {
        if !sort {
            // Fast path: find first image
            let mut reader_ref = self.reader.borrow_mut();
            let reader = reader_ref.as_mut()
                .ok_or(CbxError::Archive("Reader not available".to_string()))?;

            let mut first_image: Option<ArchiveEntry> = None;

            reader.for_each_entries(|entry, _data| {
                if is_image_file(entry.name()) {
                    first_image = Some(ArchiveEntry {
                        name: entry.name().to_string(),
                        size: entry.size(),
                        is_directory: entry.is_directory(),
                    });
                    Ok(false)  // Stop
                } else {
                    Ok(true)  // Continue
                }
            })?;

            return first_image
                .ok_or(CbxError::Archive("No images found".to_string()));
        }

        // Sorted path: collect all entries
        let entries = self.list_entries()?;
        let names: Vec<String> = entries.iter().map(|e| e.name.clone()).collect();
        let image_name = find_first_image(names.iter().map(|s| s.as_str()), true)
            .ok_or(CbxError::Archive("No images found".to_string()))?;

        entries.into_iter()
            .find(|e| e.name == image_name)
            .ok_or(CbxError::Archive("Image not found".to_string()))
    }
}
```

#### 3. Implement extract_entry

```rust
fn extract_entry(&self, entry: &ArchiveEntry) -> Result<Vec<u8>> {
    // Safety check
    if entry.size > MAX_ENTRY_SIZE {
        return Err(CbxError::Archive("Entry too large".to_string()));
    }

    // Need to recreate reader because sevenz-rust consumes it
    // This is a limitation of the library
    let mut temp_source = /* clone or recreate source */;
    let mut reader = SevenZReader::new(&mut temp_source, self.size, Password::empty())?;

    let mut extracted = None;

    reader.for_each_entries(|sz_entry, data| {
        if sz_entry.name() == entry.name {
            let mut buffer = Vec::with_capacity(sz_entry.size() as usize);
            std::io::copy(data, &mut buffer)?;
            extracted = Some(buffer);
            Ok(false)  // Stop
        } else {
            Ok(true)  // Continue
        }
    })?;

    extracted.ok_or(CbxError::Archive("Entry not found".to_string()))
}
```

### Challenge: Source Recreation

The main challenge is that `sevenz-rust` may consume the reader during iteration. We have two sub-options:

**Sub-Option A: Keep Original IStream, Clone on Each Call**
```rust
pub struct SevenZipArchiveFromStream {
    stream: IStream,  // Original IStream
    size: u64,
}

impl Archive for SevenZipArchiveFromStream {
    fn find_first_image(&self, sort: bool) -> Result<ArchiveEntry> {
        let reader = IStreamReader::new(self.stream.clone());  // Clone IStream
        let mut sz = SevenZReader::new(reader, self.size, Password::empty())?;
        // Use sz...
    }
}
```

**Performance**:
- IStream::clone() is cheap (just COM AddRef)
- Creating SevenZReader parses headers (~20-50ms for 1GB file)
- Acceptable for infrequent calls

**Sub-Option B: Store Both Source and Reader**
```rust
pub struct SevenZipArchiveFromStream<R: Read + Seek> {
    source: R,  // Original source (for recreation)
    reader: RefCell<Option<SevenZReader<&mut R>>>,  // ← Problem: can't borrow self.source as &mut
}
```

**Problem**: Can't have both immutable reference to `self` and mutable reference to `self.source`.

**Solution**: Use `RefCell` for source too:

```rust
pub struct SevenZipArchiveFromStream<R: Read + Seek> {
    source: RefCell<R>,
    size: u64,
}

impl<R: Read + Seek> Archive for SevenZipArchiveFromStream<R> {
    fn find_first_image(&self, sort: bool) -> Result<ArchiveEntry> {
        let mut source = self.source.borrow_mut();
        source.seek(SeekFrom::Start(0))?;

        let mut reader = SevenZReader::new(&mut *source, self.size, Password::empty())?;

        // Use reader...
    }
}
```

---

## Recommended Implementation Plan

### Phase 1: Simple RefCell with IStream Clone (Quick Win)

```rust
pub struct SevenZipArchiveFromStream {
    stream: IStream,
    size: u64,
}

impl Archive for SevenZipArchiveFromStream {
    fn find_first_image(&self, sort: bool) -> Result<ArchiveEntry> {
        let reader = IStreamReader::new(self.stream.clone());
        let mut sz = SevenZReader::new(reader, self.size, Password::empty())?;

        if !sort {
            let mut first = None;
            sz.for_each_entries(|entry, _| {
                if is_image_file(entry.name()) {
                    first = Some(ArchiveEntry { /* ... */ });
                    Ok(false)
                } else {
                    Ok(true)
                }
            })?;
            return first.ok_or(...);
        }

        // Sorted path...
    }

    fn extract_entry(&self, entry: &ArchiveEntry) -> Result<Vec<u8>> {
        let reader = IStreamReader::new(self.stream.clone());
        let mut sz = SevenZReader::new(reader, self.size, Password::empty())?;

        let mut data = None;
        sz.for_each_entries(|sz_entry, reader| {
            if sz_entry.name() == entry.name {
                let mut buf = Vec::new();
                std::io::copy(reader, &mut buf)?;
                data = Some(buf);
                Ok(false)
            } else {
                Ok(true)
            }
        })?;

        data.ok_or(...)
    }
}
```

**Performance Analysis**:
- Each call creates new `SevenZReader`
- Parses 7z headers each time (~20-50ms)
- For typical usage (1-2 calls per thumbnail), this is acceptable

**Trade-off**:
- ➕ Simple implementation
- ➕ No complex RefCell juggling
- ➕ Works with current architecture
- ➖ Slightly slower than caching reader
- ➖ But still MUCH faster than loading entire archive to memory!

### Phase 2: Optimize with Cached Reader (Future)

Once Phase 1 is validated, optimize by caching the reader:

```rust
pub struct SevenZipArchiveFromStream {
    stream: RefCell<IStream>,
    reader_cache: RefCell<Option<SevenZReader<IStreamReader>>>,
    size: u64,
}
```

This requires more complex lifetime management but eliminates repeated header parsing.

---

## Performance Estimates

### Current (Memory-Based)
```
1GB 7z archive:
1. Load to memory:    ~3000ms
2. Parse headers:     ~50ms
3. Find image:        ~10ms
4. Extract:           ~50ms
────────────────────────────
Total:                ~3110ms
Memory:               1GB
```

### Phase 1 (Streaming with Reader Recreation)
```
1GB 7z archive:
1. Create reader:     ~50ms (parse headers)
2. Find image:        ~10ms (streaming)
3. Create reader:     ~50ms (parse headers again)
4. Extract:           ~50ms (streaming)
────────────────────────────
Total:                ~160ms
Memory:               ~2MB
```

**Improvement**: **19.4x faster**, **500x less memory**

### Phase 2 (Streaming with Cached Reader)
```
1GB 7z archive:
1. Create reader:     ~50ms (parse headers once)
2. Find image:        ~10ms
3. Extract:           ~50ms (use cached reader)
────────────────────────────
Total:                ~110ms
Memory:               ~2MB
```

**Improvement**: **28.3x faster**, **500x less memory**

---

## Implementation Checklist

- [ ] Implement `SevenZipArchiveFromStream` with IStream clone approach
- [ ] Update `open_archive_from_stream()` to use new handler
- [ ] Add unit tests for 7z streaming
- [ ] Benchmark with real 7z files
- [ ] Compare vs memory-based approach
- [ ] Verify memory usage is reduced
- [ ] Update documentation
- [ ] (Optional) Implement Phase 2 optimization with cached reader

---

## Conclusion

7z streaming optimization is **definitely achievable** with minimal code changes:

1. **Use RefCell + IStream clone approach** (simplest)
2. **Expected improvement**: 19-28x faster, 500x less memory
3. **Implementation time**: 2-4 hours
4. **Risk**: Low (isolated change)

**Should we proceed with implementation?** ✅ **YES!**

The performance gain justifies the effort, especially for users with large 7z archives.
