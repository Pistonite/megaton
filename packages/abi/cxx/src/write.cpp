#include "write.h"
#include <string.h>
#include <nn/fs/fs_directories.h>
#include <nn/fs/fs_directories.h>
#include <switch/types.h>


namespace fs {
    uint64_t open_dir(rust::Str path) {
        nn::Result r;
        nn::fs::DirectoryHandle handle;

        // add null terminator to path
        char p[50] = {0};
        strcpy(p, path.data());
        p[path.length()] = 0;

        r = nn::fs::OpenDirectory(&handle, p, nn::fs::OpenDirectoryMode_All);

        if(r.IsFailure()){
            botw::tcp::sendf("Opening root directory failed!\n");  
            return -1;
        }

        s64 dirCount;
        r = nn::fs::GetDirectoryEntryCount(&dirCount, handle);

        if(r.IsFailure()){
            botw::tcp::sendf("Opening get dir entry count failed!\n");  
            return -1;
        } 
        botw::tcp::sendf("Calling open directory succeeded! Handle: %d. #Entries in dir: %d\n", handle._internal, dirCount);  
        return handle._internal;
    }

    // TODO: Have helpers for all the fs ops in here, and rename file to fs.cpp?
    uint64_t open_file(rust::Str path, int flags, uint32_t mode) {
        return 0;
    }

    void close_file(uint64_t fd) {
        
    }

    uint64_t write_file(uint64_t fd, int64_t pos, uint8_t* buf, usize len) {
        return len;
    }
}

