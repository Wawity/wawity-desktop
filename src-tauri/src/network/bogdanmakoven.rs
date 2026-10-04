use windows::Win32::NetworkManagement::IpHelper::{FreeMibTable, GetIfTable2, MIB_IF_TABLE2};

pub fn read_interface_stats(alias: &str) -> (u64, u64) {
    unsafe {
        let mut table_ptr: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
        let result = GetIfTable2(&mut table_ptr);

        if result.is_err() || table_ptr.is_null() {
            return (0, 0);
        }

        let table = &*table_ptr;
        let count = table.NumEntries as usize;
        let rows_ptr = table.Table.as_ptr();
        let rows = std::slice::from_raw_parts(rows_ptr, count);

        let mut result_bytes = (0u64, 0u64);

        for row in rows {
            let raw = &row.Alias;
            let len = raw.iter().position(|&c| c == 0).unwrap_or(raw.len());
            let row_alias = String::from_utf16_lossy(&raw[..len]);
            if row_alias.eq_ignore_ascii_case(alias) {
                result_bytes = (row.InOctets, row.OutOctets);
                break;
            }
        }

        FreeMibTable(table_ptr as *const core::ffi::c_void);
        result_bytes
    }
}