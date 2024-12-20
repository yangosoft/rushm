use std::ffi::CString;
use std::marker::PhantomData;
use std::ptr;

const INVALID_FD: i32 = -1;

pub struct POSIXShm<T>
where
    T: Clone,
{
    file_path: String,
    fd_shm: i32,
    mem_size: usize,
    ptr_data: *mut libc::c_void,
    phantom: PhantomData<T>,
}

impl<T> POSIXShm<T>
where
    T: Clone,
{
    pub fn new(path: String, size: usize) -> Self {
        POSIXShm {
            file_path: path,
            fd_shm: INVALID_FD,
            mem_size: size,
            ptr_data: std::ptr::null_mut(),
            phantom: PhantomData,
        }
    }

    pub unsafe fn open(&mut self) -> Result<(), String> {
        let c_shm_name = CString::new(self.file_path.to_string()).unwrap();

        self.fd_shm = libc::shm_open(
            c_shm_name.as_ptr(),
            libc::O_CREAT | libc::O_RDWR,
            libc::S_IRUSR | libc::S_IWUSR,
        );

        if self.fd_shm > 0 {
            let ret = libc::ftruncate(self.fd_shm, self.mem_size.try_into().unwrap());
            if ret < 0 {
                return Err(String::from("Cannot truncate"));
            }
        } else {
            let e = libc::__errno_location();

            return Err(String::from(format!(
                "Error opening shared memory {} errno {}",
                self.fd_shm, *e
            )));
        }

        self.ptr_data = libc::mmap(
            ptr::null_mut(),
            self.mem_size,
            libc::PROT_WRITE,
            libc::MAP_SHARED,
            self.fd_shm,
            0,
        );

        if self.ptr_data.is_null() {
            return Err(String::from("Error, mmap failed"));
        }

        Ok(())
    }

    pub unsafe fn read(&mut self) -> Result<&T, String> {
        if self.ptr_data.is_null() {
            return Err(String::from("Error, pointer to shared memory is NULL"));
        }

        let p = self.ptr_data as *const T;

        Ok(&*p)
    }

    pub unsafe fn write(&mut self, value: T) -> Result<(), String> {
        if self.ptr_data.is_null() {
            return Err(String::from("Error, pointer to shared memory is NULL"));
        }
        let w: *mut T = self.ptr_data as *mut T;
        *w = value;
        Ok(())
    }

    pub fn get_cptr_mut(&self) -> *mut libc::c_void {
        self.ptr_data
    }

    pub fn get_as_mut(&self) -> *mut T {
        self.ptr_data as *mut T
    }

    pub unsafe fn memset(&mut self, value: i32, n: usize) -> Result<(), String> {
        if self.ptr_data.is_null() {
            return Err(String::from("Pointer to shared memory is NULL"));
        }

        libc::memset(self.ptr_data, value, n);
        Ok(())
    }
}

#[cfg(test)]
use std::mem;

#[test]
fn test_shared_memory_write_read() {
    unsafe {
        let shm_name = "test_shared_memory_file";
        let mut posix_shm = POSIXShm::<u64>::new(shm_name.to_string(), mem::size_of::<u64>());
        let ret = posix_shm.open();
        assert!(ret.is_ok());

        let mut val: u64 = 0xAA;
        let ret = posix_shm.write(val);
        assert!(ret.is_ok());

        let ret: u64 = *posix_shm.read().unwrap();
        assert_eq!(val, ret);

        let ret = posix_shm.memset(0, mem::size_of::<u64>());
        assert!(ret.is_ok());
        let ret: u64 = *posix_shm.read().unwrap();
        assert_eq!(ret, 0);

        let shm_ptr = posix_shm.get_cptr_mut();
        val = 0xFF;

        let mut cptr_mut: *mut u64 = shm_ptr as *mut u64;
        *cptr_mut = val;
        let ret: u64 = *posix_shm.read().unwrap();
        assert_eq!(ret, val);

        cptr_mut = posix_shm.get_as_mut();
        val = 42;
        *cptr_mut = val;
        let ret: u64 = *posix_shm.read().unwrap();
        assert_eq!(ret, val);
    }
}
