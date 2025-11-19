#include "write.h"
#include "/home/lorem/Documents/School/Capstone/nnheaders/include/nn/fs/fs_directories.h"
#include "/home/lorem/Documents/School/Capstone/nnheaders/include/nn/fs/fs_files.h"
#include <string.h>
#include <toolkit/tcp.hpp>
#include <switch/types.h>


// invalid conversion from 'uint64_t (*)(const int8_t*, int, int32_t)
// to               'uint64_t (*)(rust::cxxbridge1::Str, int32_t, int32_t)'
namespace fs {
    uint64_t open_dir(rust::Str path) {
        nn::Result r;
        nn::fs::DirectoryHandle handle;
        // const char* path = "rom:/";

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
    uint64_t open_file(rust::Str path, int flags, int32_t mode) {
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
        // TODO: Uncomment, fix build script to properly include nnheaders
        botw::tcp::sendf("Calling close_file!\n");
        struct nn::fs::FileHandle  f = { (u64) fd };
        nn::fs::CloseFile(f);   
    }

    uint64_t write_file(uint64_t fd, int64_t pos, uint8_t* buf, usize len) {
        botw::tcp::sendf("Calling close_file!\n");
        struct nn::fs::FileHandle f = { (u64) fd };
        // WriteFile(FileHandle handle, s64 position, void const* buffer, u64 size,
        //          WriteOption const& option);

        const struct nn::fs::WriteOption opt = {0};
        nn::Result result = nn::fs::WriteFile(f, pos, buf, len, opt);
        if(result.IsFailure()) {
            botw::tcp::sendf("Calling write_file failed with %d. Description: %d\n", result.GetInnerValueForDebug(), result.GetDescription());
            return -1; // TODO: Real error codes
        }

        return len;
    }
}

