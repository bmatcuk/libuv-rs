use crate::{FromInner, Inner, IntoInner};
use uv::{uv_random, uv_random_t};

/// Additional data stored on the request
pub(crate) struct RandomDataFields {
    random_cb: Option<Box<dyn FnMut(RandomReq, i32, Vec<u8>)>>,
}

/// Callback for uv_random
extern "C" fn uv_random_cb(
    req: *mut uv_random_t,
    status: std::os::raw::c_int,
    buf: *mut std::os::raw::c_void,
    buflen: usize,
) {
    let dataptr = crate::Req::get_data(uv_handle!(req));
    if !dataptr.is_null() {
        unsafe {
            if let super::RandomData(d) = &mut *dataptr {
                if let Some(f) = d.random_cb.as_mut() {
                    let buf = Vec::from_raw_parts(buf as _, buflen, buflen);
                    f(req.into_inner(), status as _, buf);
                }
            }
        }
    }

    // free memory
    let mut req = RandomReq::from_inner(req);
    req.destroy();
}

/// Random data request type.
pub struct RandomReq {
    req: *mut uv_random_t,
}

impl RandomReq {
    /// Create a new random request
    pub fn new(
        cb: Option<impl FnMut(RandomReq, i32, Vec<u8>) + 'static>,
    ) -> crate::Result<RandomReq> {
        let layout = std::alloc::Layout::new::<uv_random_t>();
        let req = unsafe { std::alloc::alloc(layout) as *mut uv_random_t };
        if req.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let random_cb = cb.map(|f| Box::new(f) as _);
        crate::Req::initialize_data(
            uv_handle!(req),
            super::RandomData(RandomDataFields { random_cb }),
        );

        Ok(RandomReq { req })
    }

    /// Free memory - this will be called automatically after the callback (if using the async
    /// function), or before returning the random data (if using the sync version)
    pub fn destroy(&mut self) {
        crate::Req::free_data(uv_handle!(self.req));

        let layout = std::alloc::Layout::new::<uv_random_t>();
        unsafe { std::alloc::dealloc(self.req as _, layout) };
    }
}

impl FromInner<*mut uv_random_t> for RandomReq {
    fn from_inner(req: *mut uv_random_t) -> RandomReq {
        RandomReq { req }
    }
}

impl Inner<*mut uv_random_t> for RandomReq {
    fn inner(&self) -> *mut uv_random_t {
        self.req
    }
}

impl Inner<*mut uv::uv_req_t> for RandomReq {
    fn inner(&self) -> *mut uv::uv_req_t {
        uv_handle!(self.req)
    }
}

impl From<RandomReq> for crate::Req {
    fn from(random: RandomReq) -> crate::Req {
        crate::Req::from_inner(Inner::<*mut uv::uv_req_t>::inner(&random))
    }
}

impl crate::ToReq for RandomReq {
    fn to_req(&self) -> crate::Req {
        crate::Req::from_inner(Inner::<*mut uv::uv_req_t>::inner(self))
    }
}

impl crate::ReqTrait for RandomReq {}

impl crate::Loop {
    /// Fill a buf with exactly buflen cryptographically strong random bytes acquired from the
    /// system CSPRNG. flags is reserved for future extension and must currently be 0.
    ///
    /// Short reads are not possible. When less than buflen random bytes are available, a non-zero
    /// error value is returned or passed to the callback.
    ///
    /// The asynchronous version may not ever finish when the system is low on entropy.
    ///
    /// Sources of entropy:
    ///
    ///   * Windows: RtlGenRandom
    ///     <https://docs.microsoft.com/en-us/windows/desktop/api/ntsecapi/nf-ntsecapi-rtlgenrandom>_.
    ///   * Linux, Android: getrandom(2) if available, or urandom(4) after reading from /dev/random
    ///     once, or the KERN_RANDOM sysctl(2).
    ///   * FreeBSD: getrandom(2) <https://www.freebsd.org/cgi/man.cgi?query=getrandom&sektion=2>_,
    ///     or /dev/urandom after reading from /dev/random once.
    ///   * NetBSD: KERN_ARND sysctl(3)
    ///     <https://netbsd.gw.com/cgi-bin/man-cgi?sysctl+3+NetBSD-current>_
    ///   * macOS, OpenBSD: getentropy(2) <https://man.openbsd.org/getentropy.2>_ if available, or
    ///     /dev/urandom after reading from /dev/random once.
    ///   * AIX: /dev/random.
    ///   * IBM i: /dev/urandom.
    ///   * Other UNIX: /dev/urandom after reading from /dev/random once.
    pub fn random(
        &self,
        buflen: usize,
        flags: u32,
        cb: impl FnMut(RandomReq, i32, Vec<u8>) + 'static,
    ) -> crate::Result<RandomReq> {
        let mut req = RandomReq::new(Some(cb))?;
        let mut buf = std::mem::ManuallyDrop::new(Vec::<u8>::with_capacity(buflen));
        let result = crate::uvret(unsafe {
            uv_random(
                self.into_inner(),
                req.inner(),
                buf.as_mut_ptr() as _,
                buflen,
                flags as _,
                Some(uv_random_cb as _),
            )
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Fill a buf with exactly buflen cryptographically strong random bytes acquired from the
    /// system CSPRNG. flags is reserved for future extension and must currently be 0.
    ///
    /// Short reads are not possible. When less than buflen random bytes are available, a non-zero
    /// error value is returned or passed to the callback.
    ///
    /// The synchronous version may block indefinitely when not enough entropy is available.
    ///
    /// Sources of entropy:
    ///
    ///   * Windows: RtlGenRandom
    ///     <https://docs.microsoft.com/en-us/windows/desktop/api/ntsecapi/nf-ntsecapi-rtlgenrandom>_.
    ///   * Linux, Android: getrandom(2) if available, or urandom(4) after reading from /dev/random
    ///     once, or the KERN_RANDOM sysctl(2).
    ///   * FreeBSD: getrandom(2) <https://www.freebsd.org/cgi/man.cgi?query=getrandom&sektion=2>_,
    ///     or /dev/urandom after reading from /dev/random once.
    ///   * NetBSD: KERN_ARND sysctl(3)
    ///     <https://netbsd.gw.com/cgi-bin/man-cgi?sysctl+3+NetBSD-current>_
    ///   * macOS, OpenBSD: getentropy(2) <https://man.openbsd.org/getentropy.2>_ if available, or
    ///     /dev/urandom after reading from /dev/random once.
    ///   * AIX: /dev/random.
    ///   * IBM i: /dev/urandom.
    ///   * Other UNIX: /dev/urandom after reading from /dev/random once.
    pub fn random_sync(buflen: usize, flags: u32) -> crate::Result<Vec<u8>> {
        let mut buf: Vec<u8> = Vec::with_capacity(buflen);
        unsafe { buf.set_len(buflen) };
        crate::uvret(unsafe {
            uv_random(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                buf.as_mut_ptr() as _,
                buflen,
                flags as _,
                None::<
                    unsafe extern "C" fn(
                        *mut uv_random_t,
                        std::os::raw::c_int,
                        *mut std::os::raw::c_void,
                        usize,
                    ),
                >,
            )
        })
        .map(|_| buf)
    }
}
