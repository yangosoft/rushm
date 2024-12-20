# rushm

Rust POSIX shared memory access


Example


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
