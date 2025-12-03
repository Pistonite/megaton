#include "write.h"
#include <string.h>
#include <nn/fs/fs_directories.h>
#include <nn/fs/fs_files.h>

#include <toolkit/tcp.hpp>
#include <nn/types.h>


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

    void close_dir(uint64_t fd) {
        botw::tcp::sendf("Calling close_dir!\n");
        struct nn::fs::DirectoryHandle  f = { fd };
        nn::fs::CloseDirectory(f);  
    }

    uint64_t open_file(rust::Str path, int flags, uint32_t mode) {
        botw::tcp::sendf("Calling open_file!\n");

        // add null terminator to path
        char p[50] = {0};
        strcpy(p, path.data());
        p[path.length()] = 0;
        
        struct nn::fs::FileHandle handleOut;
        nn::Result result = nn::fs::OpenFile(&handleOut, (const char*) p, mode); 
        if(result.IsFailure()) {
            botw::tcp::sendf("Calling open_file failed with %d. Description: %d\n", result.GetInnerValueForDebug(), result.GetDescription());
            return -1;
        }
    
        return handleOut._internal;
    }

    void close_file(uint64_t fd) {
        botw::tcp::sendf("Calling close_file!\n");
        struct nn::fs::FileHandle  f = { fd };
        nn::fs::CloseFile(f);   
    }

    uint64_t write_file(uint64_t fd, int64_t pos, uint8_t* buf, usize len) {
        botw::tcp::sendf("Calling close_file!\n");
        struct nn::fs::FileHandle f = { fd };

        const struct nn::fs::WriteOption opt = {0};
        nn::Result result = nn::fs::WriteFile(f, pos, buf, len, opt);
        if(result.IsFailure()) {
            botw::tcp::sendf("Calling write_file failed with %d. Description: %d\n", result.GetInnerValueForDebug(), result.GetDescription());
            return -1; // TODO: Real error codes
        }

        return len;
    }
}

