#include <cstdio>
#include <nn/os.h>
#include <nn/sm.h>
#include <nn/fs.h>
#include <nn/settings.h>

#include <megaton/user_rt.h>
#include "megacrt.h"

extern "C" {
// #include <switch/kernel/svc.h>
#include <switch/runtime/devices/fs_dev.h>
#include <switch/kernel/detect.h>
#include <switch/runtime/hosversion.h>
#include <switch/services/sm.h>
#include <switch/services/fs.h>
#include <switch/result.h>
}

// temporary - to get diagnostics while figuring out how to initialize the stuff
void crt_check_result(Result rc, const char* content) {
    if (R_SUCCEEDED(rc)) {
        return;
    }
    char pathbuf[100];
    size_t n = snprintf(pathbuf, sizeof(pathbuf), "sd:/megaton/%s/logs/crt_diag_abort.txt", _megart_module_name());
    if (n < 0 || n >= sizeof(pathbuf)) {
        __megacrt_abort(rc);
        return;
    }
    pathbuf[n] = 0;

    char contentbuf[256];
    int module = R_MODULE(rc);
    int description = R_DESCRIPTION(rc);
    n = snprintf(contentbuf, sizeof(contentbuf), "result: %04d-%04d\nerror: %s\n", 2000 + module, description, content);
    if (n < 0 || n >= sizeof(contentbuf)) {
        __megacrt_abort(rc);
        return;
    }
    contentbuf[n] = 0;

    Result rc2 = nn::fs::MountSdCard("sd");
    if (R_FAILED(rc2)) {
        __megacrt_abort(rc);
        return;
    }

    nn::fs::FileHandle file;
    rc2 = nn::fs::OpenFile(&file, pathbuf, nn::fs::OpenMode_ReadWrite);
    if (R_FAILED(rc2)) {
        rc2 = nn::fs::CreateFile(pathbuf, n);
        if (R_SUCCEEDED(rc2)) {
            rc2 = nn::fs::OpenFile(&file, pathbuf, nn::fs::OpenMode_ReadWrite);
        }
    }
    if (R_FAILED(rc2)) {
        __megacrt_abort(rc);
        return;
    }

    nn::fs::WriteFile(file, 0, contentbuf, n, nn::fs::WriteOption::CreateOption(nn::fs::WriteOptionFlag_Flush));
    nn::fs::CloseFile(file);

    __megacrt_abort(rc);
}

extern "C" u32 __megacrt_get_hos_version_for_libnx() {
    bool ams = detectMesosphere();
    nn::settings::system::FirmwareVersion fwVersion;
    Result rc = nn::settings::system::GetFirmwareVersion(&fwVersion);
    crt_check_result(rc, "failed to get firmware version");
    u32 version = MAKEHOSVERSION(fwVersion.major, fwVersion.minor, fwVersion.micro);
    if (ams) {
        return BIT(31) | version;
    } 
    return version;
}

extern "C" Handle __megacrt_get_current_thread() {
    // use the SDK to get the current thread handle, which is needed for libnx init
    auto* thread = nn::os::GetCurrentThread();
    return thread->_handle;
}

extern "C" void __megacrt_init_services() {
    
    // initialize the service manager using the SDK
    // we override libnx services to avoid creating too many connections
    // and exceeding the process limit, since we share the limit with the game
    Result rc = nn::sm::Initialize();
    crt_check_result(rc, "failed to initialize sm");

    // we are skipping time service since it requires a lot of sessions
    nn::svc::Handle tmp_handle;
    rc = nn::sm::GetServiceHandle(&tmp_handle, "fsp-srv", 7);
    crt_check_result(rc, "failed to get fs service");
    smAddOverrideHandle(smEncodeName("fsp-srv"), tmp_handle);

    rc = fsInitialize();
    crt_check_result(rc, "failed to init fs service");

    rc = fsdevMountSdmc();
    crt_check_result(rc, "failed to mount sdmc");
}
