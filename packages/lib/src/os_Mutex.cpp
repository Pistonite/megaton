#include <nn/os/os_Mutex.h>

namespace nn::os {
Mutex::~Mutex() {
    nn::os::FinalizeMutex(&this->m_Mutex);
}
}
