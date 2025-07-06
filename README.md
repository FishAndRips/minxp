# minxp

minxp is a crate that reimplements some of the Rust standard library over Win32
for `no-std` crates. This was made for Windows versions that are no longer
supported by the Rust Standard Library, such as Windows 7.

minxp tries to maintain the same function signatures. Note, however, that some
types in `minxp`, like `minxp::path::PathBuf`, cannot be used in functions that
expect the equivalent Rust Standard Library types, like `std::path::PathBuf`.

## Compatibility

All functions are guaranteed to work on Windows XP, forever.

Currently, minxp also works on Windows 2000 on a tentative basis. This is not
promised to be the case for all time, however; it may be dropped at a later
point if supporting it would greatly impede development.

It is highly recommended to use the `*-pc-windows-gnu` targets and/or toolchains
to avoid/reduce headaches with getting executables to run on older versions of
Windows.

### Different implementation functions

A function may have different implementations depending on if functions are
available, however. For example, `minxp::fs::canonicalize` will use the
`GetFinalPathNameByHandleW` function on newer Windows versions, while it will
use `GetFullPathNameW` on older versions. If there are differences, the function
documentation will have that difference described.

All of these checks are performed once at runtime by checking for the presence
of functions and caching the result.

### Unicode on older Windows

When possible, wide functions are used to avoid codepage issues. This entails
re-encoding everything as UTF-16 with an additional null terminator at the end.

Standard I/O, however, uses 8-bit encoding, and it will override the console's
codepage with `SetConsoleOutputCP(CP_UTF8)`.

Note that, for Command Prompt (cmd.exe), you may need to configure it to use a
font that supports Unicode, such as "Lucida Console TrueType". However, even
Windows 2000 is confirmed to support UTF-8 console output with the proper setup.

### Wine

minxp is not tested on Wine, a compatibility layer for running Windows programs
on non-Windows operating systems.

No guarantee is made that a function will work as intended on Wine, crash the
process, or cause velociraptors to eject from your PC on execution. The same
applies to other compatibility layers, such as Proton.

# What is provided

The following modules are provided:

| Module    | Coverage* | Notes                                                 |
|-----------|-----------|-------------------------------------------------------|
| `env`     | Full      |                                                       |
| `ffi`     | Partial   | Missing some trait impls, `Component`                 |
| `fs`      | Partial   | Some functions not implemented; uses raw timestamps   |
| `io`      | Partial   | Partial impl of `Read`, `Write`, `Stdout`, & `Stderr` |
| `path`    | Partial   | Missing some trait impls                              |
| `process` | Partial   | Only `abort` and `exit` for now                       |

\* Only includes what's exclusively in `std`; types in `alloc` and `core` will
   not be (re-)implemented, as there is no need. If something is missing, try
   looking in there, first.

Note that Rust's Standard Library may eventually have more things in it. As
such, this is only guaranteed to be accurate as of **1.87.0**.

## Planned

This is being developed on an as-needed basis for other FishAndRips projects. At
the moment, this is what is expected:

* More `io`, `fs`, and `ffi`
* `net`
* `process`
* `thread`
* `time`

As previously stated, this only includes what is exclusive to `std`. Some things
are present in `core`. For example, `std::net::SocketAddr` is also available as
`core::net::SocketAddr`.

# Notes

Here are a few other things to note when using this project.

## minxp is not a complete standard library replacement

minxp provides functions and types, but it does not provide everything else.

To use minxp in a crate without the Rust standard library, you need a global
allocator for `alloc` types to work. Also, for standalone executables, you
need `WinMain`. The [min32] crate provides all of this. Here is an example of
a working program:

```rust
#![no_std]
#![no_main]

use min32::winmain;
use minxp::println;

#[winmain]
fn main() {
    println!("Hello world!");
}
```

[min32]: (https://github.com/FishAndRips/min32)

## Regarding deprecated functions

Functions that are deprecated by the Rust Standard Library won't be implemented
UNLESS they were previously implemented on minxp, in which case they will also
be marked as deprecated but never removed.

Functions that are not deprecated by the Rust Standard Library but are on minxp
are only temporarily marked as such, as their functionality is available, but
they cannot currently match the Rust standard library implementation. You can
use them now, but they may be changed WITHOUT a major version bump, and thus
anything that was using them will probably break.

## Regarding unstable functions

Functions present in the Rust Standard Library on non-stable versions of Rust
are not going to be implemented until they are on, at least, the beta channel.

## Regarding non-Windows targets

Non-Windows targets are not supported, although cross-compiling from non-Windows
hosts to a Windows target shouldn't have any issues if it also works with the
regular standard library.

You can also conditionally use minxp. For example, you can use feature gates,
though this probably isn't the most useful example:

```rust
#![no_std] // prevents bringing in the standard library prelude
#![cfg_attr(feature = "minxp", no_main)]

#[cfg(not(feature = "minxp"))]
extern crate std;

#[cfg(feature = "minxp")]
extern crate minxp as std;

#[cfg_attr(feature = "minxp", min32::winmain)]
fn main() {
    std::println!("Hello world!");
}
```

# License

minxp is licensed under version 3 of the GNU General Public License as published
by the Free Software Foundation in 2007. It is unavailable under any other
license at this time, including any other version of the GNU GPL.
