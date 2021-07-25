//=============================================
// Using People License
// Copyright (c) 2020-2021 GOSCPS 保留所有权利.
//=============================================
// 版本:洪武

use alloc::boxed::Box;
use alloc::vec;
use alloc::prelude::v1::Vec; 
use guuid;
use core::convert::TryInto;

/// 文件系统版本号
const VERSION_MARK : u64 = 1;

/// 超级块
struct SuperBlock{
    /// 文件系统版本号
    /// 
    /// 大端序
    pub version : u64,

    /// 文件系统Guuid
    pub guuid : guuid::Guuid,

    /// 文件系统空间长度(包括文件自身占用)
    /// 
    /// 单位字节。大端序
    pub length : u64,

    /// 磁盘占用map地址
    pub map_address : u64,

    /// 磁盘占用map长度
    pub map_length : u64,
}

/// 目录结构
struct Directory{
    /// 目录名称
    /// 
    /// UTF-8
    pub name : [u8;256],

    /// 目录项的索引
    /// 
    /// 大端序
    pub index_data_lba : u64,
}

/// 文件结构
struct File{
    /// 文件名称
    /// 
    /// UTF-8
    pub name : [u8;256],

    /// 文件大小
    /// 
    /// 大端序
    pub size : u64,

    /// 文件索引
    /// 
    /// 大端序
    pub  data_index_lba : u64,
}

/// 数据块结构
/// 
/// 大端序
struct DataBlock{
    /// 上一个数据块的索引
    /// 
    /// 为0则视为第一个数据块
    pub prev_block_index : u64,

    /// 下一个数据块的索引
    /// 
    /// 为0则视为最后一个数据块
    pub next_block_index : u64,

    /// 当前数据块长度
    pub size : u64,

    // ... 数据块内容
}

/// 目录项类型
/// 
/// 大端序
#[repr(u64)]
enum DirectoryItemType{
    File = 0,
    Directory = 1,
}

/// 目录项枚举
enum DirectoryEnum{   
    /// 文件
    File(File),
    /// 目录
    Directory(Directory),
}

/// 目录项
struct DirectoryItem{
    /// 目录项类型
    pub item_type : DirectoryItemType,
    /// 目录项索引
    pub item_lba : u64,
}

/// 大明属文件系统 - v1 洪武
pub struct HongWu{
    /// 操作接口
    operator : Box<dyn crate::DiskOperator>,
    /// 超级块
    super_block : Option<Box<SuperBlock>>,
}

impl HongWu{

    /// 构造函数
    pub fn new(interface : Box<dyn crate::DiskOperator>) -> Box<HongWu>{
        // 进行接口检查
        // 逻辑块大小至少为 512 字节
        if (*interface).block_size() < 512{
            panic!("interface.block_size() < 512!");
        }
        // 逻辑块数量至少为16
        if (*interface).block_count() < 16{
            panic!("interface.block_num() < 16!");
        }

        // 构造一个新的大明属文件系统
        Box::new(
            HongWu{
                operator : interface,
                super_block : None,
            }
        )
    }

    /// 格式化
    pub fn format(&mut self,guuid : guuid::Guuid){
        // 构造超级块
        let super_block = SuperBlock{
            version : VERSION_MARK,
            guuid : guuid,
            length : self.operator.block_size() * self.operator.block_count(),
            map_address : 1,
            map_length : (self.operator.block_count() / 8) / self.operator.block_size() + 1
        };

        self.super_block = Some(Box::new(super_block));

        // 写入超级块
        // lba 0
        let mut datas : Vec<u8> = Vec::new();

        datas.append(&mut self.super_block.as_mut().unwrap().version.to_be_bytes().to_vec());
        datas.append(&mut self.super_block.as_mut().unwrap().guuid.to_bytes().to_vec());
        datas.append(&mut self.super_block.as_mut().unwrap().length.to_be_bytes().to_vec());
        datas.append(&mut self.super_block.as_mut().unwrap().map_address.to_be_bytes().to_vec());
        datas.append(&mut self.super_block.as_mut().unwrap().map_length.to_be_bytes().to_vec());

        self.operator.write(0,datas.as_slice()).unwrap();
        datas.clear();

        // 初始化map
        datas.append(&mut vec![0u8;self.operator.block_size().try_into().unwrap()]);

        {
            let current = 0u64;

            while current < self.super_block.as_mut().unwrap().map_length {
                // 初始化为0
                self.operator.write(self.super_block.as_mut().unwrap().map_address + current,datas.as_slice()).unwrap();
            }

            // 标记map的0和1位为占用状态
            // 为0表示未占用
            // 为1表示占用
            // 从低位开始标记
            self.operator.write(self.super_block.as_mut().unwrap().map_address,&mut vec![0u8 & 0b00000011u8]).unwrap();
        }

        // 构造根目录

        // 格式化完毕
    }

    /// 获取一个空闲区块
    pub fn get_free_block(&mut self) -> Option<u64>{
        // 空闲lba索引
        let mut free_blocks;

        // 检查map的每一位
        // 一般认为，如果map的某一位为0，则说明该位置为空闲区块
        for current in 0..self.super_block.as_mut().unwrap().length{
            // 读取区块
            let mut buffer : Vec<u8> = vec![0u8;self.operator.block_size().try_into().unwrap()];
            self.operator.read(self.super_block.as_mut().unwrap().map_address + current,&mut buffer).unwrap();

            // 检查是否有某位为0
            for index in 0..buffer.len(){
                let mut index_u8 = u8::from_be(buffer[index]);
                let trailing_zeros = index_u8.trailing_zeros();

                // 检查是否有前导0
                if trailing_zeros != 0{
                    // 前导0存在
                    // 说明有扇区空闲

                    // 获取此扇区的索引
                    free_blocks = current * self.operator.block_size() * 8;

                    // 加上扇区内的偏移
                    free_blocks = free_blocks + (index as u64 * 8);

                    // 获取扇区偏移内的位偏移
                    free_blocks = free_blocks + (8 - trailing_zeros as u64);

                    // 设置此位为占用
                    index_u8 = (index_u8 << 1) & 1u8;
                    buffer[index] = index_u8.to_be();

                    // 检查是否在范围内
                    if free_blocks >= self.operator.block_count(){
                        // 超出范围
                        return None;
                    }

                    // 正确的索引
                    // 写入map
                    self.operator.write(self.super_block.as_mut().unwrap().map_address + current,&mut buffer).unwrap();
                    // 返回
                    return Some(free_blocks);
                }
            }

        }

        None
    }

    /// 创建目录
    /// 
    /// 自己在map中标记占用
    /// 
    /// 返回创建目录所在地址
    fn create_directory(&mut self,name : [u8;256]) -> Option<u64>{
        // 查找空闲map
        let dic = self.get_free_block()?;

        // 创建目录
        let directory = Directory{
            name : name,
            index_data_lba : 0,
        };

        // 构造
        let mut datas : Vec<u8> = Vec::new();
        datas.append(&mut directory.name.to_vec());
        datas.append(&mut directory.index_data_lba.to_be_bytes().to_vec());

        // 写入目录
        self.operator.write(dic,&mut datas).unwrap();

        Some(dic)
    }




}
