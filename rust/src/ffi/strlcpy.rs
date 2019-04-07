#![allow(
    dead_code,
    mutable_transmutes,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_mut
)]
/// Ported from OpenBSD: https://cvsweb.openbsd.org/cgi-bin/cvsweb/~checkout~/src/lib/libc/string/strlcpy.c?rev=1.16&content-type=text/plain
extern crate libc;

/*
 * Copy string src to buffer dst of size dsize.  At most dsize-1
 * chars will be copied.  Always NUL terminates (unless dsize == 0).
 * Returns strlen(src); if retval >= dsize, truncation occurred.
 */
// Temporary workaround for nightly feature.
fn wrapping_offset_from<T>(this: *const T, origin: *const T) -> isize
where
    T: Sized,
{
    let pointee_size = std::mem::size_of::<T>();
    assert!(0 < pointee_size && pointee_size <= isize::max_value() as usize);

    let d = isize::wrapping_sub(this as _, origin as _);
    d.wrapping_div(pointee_size as _)
}

pub unsafe fn strlcpy(
    mut dst: *mut libc::c_char,
    mut src: *const libc::c_char,
    mut dsize: libc::size_t,
) -> libc::size_t {
    let mut osrc: *const libc::c_char = src;
    let mut nleft: libc::size_t = dsize;
    if nleft != 0 as libc::size_t {
        loop {
            nleft = nleft.wrapping_sub(1);
            if nleft == 0 as libc::size_t {
                break;
            }
            let fresh1 = dst;
            dst = dst.offset(1);
            let fresh0 = src;
            src = src.offset(1);
            *fresh1 = *fresh0;
            if *fresh1 == 0 {
                break;
            }
        }
    }
    if nleft == 0 as libc::size_t {
        if dsize != 0 as libc::size_t {
            *dst = '\u{0}' as i32 as libc::c_char
        }
        loop {
            let fresh2 = src;
            src = src.offset(1);
            if 0 == *fresh2 {
                break;
            }
        }
    }
    (wrapping_offset_from(src, osrc) as libc::c_long - 1 as libc::c_long) as libc::size_t
}
