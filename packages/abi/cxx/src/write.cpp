#include "write.h"
#include "../../../../../nnheaders/include/nn/fs/fs_files.h"

#include <toolkit/tcp.hpp>
#include <switch/types.h>
namespace fs {
    // TODO: Have helpers for all the fs ops in here, and rename file to fs.cpp?
    uint64_t open_file(const int8_t *name, int flags, int32_t mode) {
        // TODO: Uncomment, fix build script to properly include nnheaders
        botw::tcp::sendf("Calling open_file!\n");
        
        // nn::fs::FileHandle handleOut;
        // nn::Result result = OpenFile(&handleOut, (const char*) name, mode); 
        // if(result.IsSuccess()) return handleOut._internal;
        
        return 0;
    }
}

