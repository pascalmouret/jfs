use crate::blockmap::BlockMap;
use crate::structure::inode::Inode;
use crate::superblock::SuperBlock;
use crate::structure::inode_table::InodeTable;

mod inode_table;
mod inode;

struct Structure<INODE: Inode> {
    super_block: SuperBlock,
    block_map: BlockMap,
    inode_table: InodeTable<INODE>,
}
