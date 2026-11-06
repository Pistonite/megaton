#pragma once

#include <nn/svc.h>

namespace nn::os {

class ProcessId {

public:
    u64 id;
};

}

namespace nn::sm {

Result Initialize();
// todo: return type
Result Finalize();
Result GetServiceHandle(nn::svc::Handle* handleOut, const char* name, size_t nameLen);
Result RegisterService(nn::svc::Handle* handle, const char* name, size_t nameLen, int, bool);
Result UnregisterService(const char* name, ulong);
Result InitializeForManager();
void FinalizeForManager();
Result RegisterProcess(nn::os::ProcessId process, const void*, size_t, const void*, size_t);
Result UnregisterProcess(nn::os::ProcessId process);




}
