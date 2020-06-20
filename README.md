# Oggetto
Oggetto is Reed-Solomon applied to an object storage system to offer resiliency and high availability of data


This is intended to be an high level library based on top of reed-solomon-erasure rust library

The object storage system want to:

- Create an high way of managing the File structure and Chunk structure as an object storage system
- Abstract the lower level of file manage problem, so that the chunks/blocks/files can stay on remote/local/network storage
- Implent POSIX file function
- Work with FUSE library
- Fix broken bit problem in file thanks to Reed Solomon encoding


# What We miss

- [ ] Posix
- [ ] Intelligent Allocation of file
- [ ] Network File allocator
- [X] Dumb allocation of file
- [X] Create File function
- [ ] Object storage functionality
- [ ] Fuse access