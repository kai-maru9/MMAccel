use crate::*;

pub fn image_import_desc(
    base_addr: usize,
    target: &[u8],
) -> windows::Result<IMAGE_IMPORT_DESCRIPTOR> {
    unsafe {
        let mut size = 0;
        let mut img_desc_ptr = ImageDirectoryEntryToData(
            base_addr as _,
            1,
            IMAGE_DIRECTORY_ENTRY::IMAGE_DIRECTORY_ENTRY_IMPORT,
            &mut size,
        ) as *mut IMAGE_IMPORT_DESCRIPTOR;
        if img_desc_ptr.is_null() {
            return Err(get_last_error().into());
        }
        loop {
            let img_desc = &*img_desc_ptr;
            if img_desc.Name == 0 {
                return Err(get_last_error().into());
            }
            let p = (base_addr + img_desc.Name as usize) as *const u8;
            let name = std::slice::from_raw_parts(p, target.len());
            let name = name
                .iter()
                .map(|c| c.to_ascii_lowercase())
                .collect::<Vec<_>>();
            if name.iter().eq(target) {
                break;
            }
            img_desc_ptr = img_desc_ptr.offset(1);
        }
        Ok(*img_desc_ptr)
    }
}

pub fn inject_functions(
    base_addr: usize,
    img_desc: &IMAGE_IMPORT_DESCRIPTOR,
    functions: &[(&[u8], u64)],
) -> windows::Result<()> {
    unsafe {
        let mut iat_ptr = (base_addr + img_desc.FirstThunk as usize) as *mut IMAGE_THUNK_DATA64;
        let mut int_ptr =
            (base_addr + img_desc.Anonymous.OriginalFirstThunk as usize) as *mut IMAGE_THUNK_DATA64;
        while iat_ptr.as_ref().unwrap().u1.Function != 0 {
            let mut iat = &mut *iat_ptr;
            let int = &*int_ptr;
            if (int.u1.Ordinal & 0x8000000000000000) != 0 {
                continue;
            }
            let name_ptr =
                (base_addr + int.u1.AddressOfData as usize) as *const IMAGE_IMPORT_BY_NAME;
            for &(function_name, fp) in functions.iter() {
                let name = std::slice::from_raw_parts(
                    name_ptr.as_ref().unwrap().Name.as_ptr() as *const u8,
                    function_name.len(),
                );
                let mut old_type = PAGE_TYPE::default();
                if name.iter().eq(function_name) {
                    VirtualProtect(
                        (&mut iat.u1.Function) as *mut _ as _,
                        std::mem::size_of::<u64>() as _,
                        PAGE_TYPE::PAGE_READWRITE,
                        &mut old_type,
                    );
                    iat.u1.Function = fp;
                    VirtualProtect(
                        (&mut iat.u1.Function) as *mut _ as _,
                        std::mem::size_of::<u64>() as _,
                        old_type,
                        &mut old_type,
                    );
                }
            }
            iat_ptr = iat_ptr.offset(1);
            int_ptr = int_ptr.offset(1);
        }
    }
    Ok(())
}
