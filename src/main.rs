use simpledb::file::block_id::BlockId;
use simpledb::file::file_manager::FileMgr;
use simpledb::file::page::Page;

fn main() -> anyhow::Result<()> {
    let block_id = BlockId::default();
    println!("{}", block_id);

    let page = Page::default();
    println!("{}", page);

    let file_mgr = FileMgr::new("hi", 10)?;
    dbg!(file_mgr);

    Ok(())
}
