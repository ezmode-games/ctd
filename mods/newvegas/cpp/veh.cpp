// Vectored Exception Handler for crash capture (32-bit version).

#include <Windows.h>
#include <DbgHelp.h>
#include <Psapi.h>

#include <sstream>
#include <string>

#include "ctd-newvegas/src/lib.rs.h"  // CXX-generated Rust interface
#include "veh.hpp"

#pragma comment(lib, "DbgHelp.lib")

namespace {

// Check if an exception code is fatal
bool is_fatal_exception(DWORD code) {
    switch (code) {
        case EXCEPTION_ACCESS_VIOLATION:
        case EXCEPTION_STACK_OVERFLOW:
        case EXCEPTION_ILLEGAL_INSTRUCTION:
        case EXCEPTION_INT_DIVIDE_BY_ZERO:
        case EXCEPTION_INT_OVERFLOW:
        case EXCEPTION_PRIV_INSTRUCTION:
        case EXCEPTION_IN_PAGE_ERROR:
        case EXCEPTION_INVALID_HANDLE:
        case 0xC0000374:  // HEAP_CORRUPTION
        case 0xC0000409:  // STACK_BUFFER_OVERRUN
            return true;
        default:
            return false;
    }
}

// Get module name for an address
std::string get_module_name(void* address) {
    HMODULE module;
    if (GetModuleHandleExA(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            static_cast<LPCSTR>(address),
            &module)) {
        char name[MAX_PATH];
        if (GetModuleFileNameA(module, name, MAX_PATH)) {
            std::string path(name);
            auto pos = path.find_last_of("\\/");
            return (pos != std::string::npos) ? path.substr(pos + 1) : path;
        }
    }
    return "unknown";
}

// Get module base address
uintptr_t get_module_base(void* address) {
    HMODULE module;
    if (GetModuleHandleExA(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            static_cast<LPCSTR>(address),
            &module)) {
        return reinterpret_cast<uintptr_t>(module);
    }
    return 0;
}

// Walk the stack and build a trace (32-bit version)
std::string capture_stack_trace(CONTEXT* context) {
    std::ostringstream trace;

    HANDLE process = GetCurrentProcess();
    HANDLE thread = GetCurrentThread();

    // Initialize symbol handler
    SymSetOptions(SYMOPT_LOAD_LINES | SYMOPT_UNDNAME);
    SymInitialize(process, nullptr, TRUE);

    STACKFRAME frame = {};
    frame.AddrPC.Offset = context->Eip;
    frame.AddrPC.Mode = AddrModeFlat;
    frame.AddrFrame.Offset = context->Ebp;
    frame.AddrFrame.Mode = AddrModeFlat;
    frame.AddrStack.Offset = context->Esp;
    frame.AddrStack.Mode = AddrModeFlat;

    CONTEXT ctx = *context;

    for (int i = 0; i < 64; ++i) {
        if (!StackWalk(
                IMAGE_FILE_MACHINE_I386,
                process,
                thread,
                &frame,
                &ctx,
                nullptr,
                SymFunctionTableAccess,
                SymGetModuleBase,
                nullptr)) {
            break;
        }

        if (frame.AddrPC.Offset == 0) {
            break;
        }

        void* addr = reinterpret_cast<void*>(frame.AddrPC.Offset);
        std::string module = get_module_name(addr);
        uintptr_t base = get_module_base(addr);
        uintptr_t offset = frame.AddrPC.Offset - base;

        trace << "[" << i << "] " << module << "+0x" << std::hex << offset
              << " (0x" << frame.AddrPC.Offset << ")\n";
    }

    SymCleanup(process);

    return trace.str();
}

// The VEH handler callback
LONG WINAPI veh_handler(PEXCEPTION_POINTERS info) {
    if (!info || !info->ExceptionRecord) {
        return EXCEPTION_CONTINUE_SEARCH;
    }

    DWORD code = info->ExceptionRecord->ExceptionCode;

    if (!is_fatal_exception(code)) {
        return EXCEPTION_CONTINUE_SEARCH;
    }

    // Build exception data
    ctd::ExceptionData data;
    data.code = code;
    data.address = reinterpret_cast<uint64_t>(info->ExceptionRecord->ExceptionAddress);
    data.stack_trace = rust::String(capture_stack_trace(info->ContextRecord));
    data.faulting_module = rust::String(get_module_name(info->ExceptionRecord->ExceptionAddress));

    // Hand off to Rust (fire-and-forget)
    ctd::handle_crash(data);

    return EXCEPTION_CONTINUE_SEARCH;
}

}  // namespace

namespace ctd {

void register_veh_handler() {
    AddVectoredExceptionHandler(1, veh_handler);
}

}  // namespace ctd
