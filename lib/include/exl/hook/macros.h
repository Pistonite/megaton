
#define _HOOK_STATIC_CALLBACK_ASSERT()                                         \
    static_assert(!std::is_member_function_pointer_v<CallbackFuncPtr<>>,       \
                  "Callback method must be static!")
