# orthanc-import
Upload DICOM files to an Orthanc server via the REST API

## Comparsion to `OrthancImport.py`
### Pros
- Multithreaded (Useful for large imports)
### Cons
- Only DICOM files are supported. Cannot upload compressed archives (ZIP, GZip, BZip, etc.) unless decompression is handled by the Orthanc server itself.