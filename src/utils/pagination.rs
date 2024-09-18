// pagination
//
// return (offset, limit)
pub fn compute(page: u32, page_size: u32) -> (u32, u32) {
    if page == 0 {
        return (0, page_size);
    }

    ((page - 1) * page_size, page_size)
}
