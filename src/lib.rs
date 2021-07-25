//=============================================
// Using People License
// Copyright (c) 2020-2021 GOSCPS 保留所有权利.
//=============================================
// 仅仅启用alloc
#![no_std]

extern crate alloc;

#[cfg(test)]
mod tests;

// 引入子版本
pub mod hong_wu;

// 字符串
use alloc::prelude::v1::String;

/// 预定义的一些错误类型
#[derive(Clone,Debug)]
pub enum ErrorType{
    /// 索引超过范围
    OutOfLbaRange,

    /// 写入的数据太长
    WriteTooMuch,

    /// 未定义异常 使用字符串报告
    UndefinedError(String),
}

/// 磁盘操作接口
pub trait DiskOperator {
    /// 返回逻辑块大小
    fn block_size(&self) -> u64;

    /// 返回逻辑块数量
    fn block_count(&self) -> u64;

    /// 读操作
    /// 
    /// - lba 为逻辑块号
    /// - buffer 为读取数据缓冲区
    /// 
    /// 返回实际读取的字节数
    fn read(&mut self,lba : u64, buffer : &[u8]) -> Result<u64,ErrorType>;

    /// 写操作
    /// 
    /// - lba 为逻辑块号
    /// - buffer 为写入数据缓冲区
    fn write(&mut self,lba : u64, buffer : &[u8]) -> Result<(),ErrorType>;
}



