//! 高位优先（MSB-first）的比特流读取器与前缀查表 Huffman 解码器。

use crate::tables::Codebook;
use crate::{fail, Result};

#[derive(Clone)]
pub struct BitReader<'a> {
    data: &'a [u8],
    pub pos: usize,
    pub end: usize,
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0, end: data.len() * 8 }
    }

    pub fn with_range(data: &'a [u8], bit_pos: usize, bit_end: usize) -> Result<Self> {
        if bit_pos > bit_end || bit_end > data.len() * 8 {
            return fail(format!(
                "bits {bit_pos}..{bit_end} lie outside {} bytes",
                data.len()
            ));
        }
        Ok(Self { data, pos: bit_pos, end: bit_end })
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.end.saturating_sub(self.pos)
    }

    #[inline]
    pub fn read(&mut self, n: u32) -> Result<u32> {
        let count = n as usize;
        if self.pos + count > self.end {
            return fail("read past the end of the field");
        }
        if count == 0 {
            return Ok(0);
        }
        let mut value = 0u32;
        let mut rem = count;
        if self.pos & 7 == 0 && rem >= 8 {
            while rem >= 8 {
                value = (value << 8) | u32::from(self.data[self.pos >> 3]);
                self.pos += 8;
                rem -= 8;
            }
        }
        while rem > 0 {
            let bit_in_byte = self.pos & 7;
            let take = rem.min(8 - bit_in_byte);
            let byte = u32::from(self.data[self.pos >> 3]);
            let shift = 8 - bit_in_byte - take;
            let mask = (1u32 << take) - 1;
            value = (value << take) | ((byte >> shift) & mask);
            self.pos += take;
            rem -= take;
        }
        Ok(value)
    }

    /// 预读接下来最多 n 位（不推进 pos），越界部分补零。
    #[inline]
    pub fn peek(&self, n: u32) -> u32 {
        let count = n as usize;
        if count == 0 {
            return 0;
        }
        let avail = self.end.saturating_sub(self.pos).min(count);
        let mut value = 0u32;
        let mut pos = self.pos;
        let mut rem = avail;
        while rem > 0 {
            let bit_in_byte = pos & 7;
            let take = rem.min(8 - bit_in_byte);
            let byte = u32::from(self.data[pos >> 3]);
            let shift = 8 - bit_in_byte - take;
            let mask = (1u32 << take) - 1;
            value = (value << take) | ((byte >> shift) & mask);
            pos += take;
            rem -= take;
        }
        value << (count - avail)
    }

    #[inline]
    pub fn flag(&mut self) -> Result<bool> {
        Ok(self.read(1)? != 0)
    }

    /// 读取 n 位二进制补码有符号整数。
    #[inline]
    pub fn signed(&mut self, n: u32) -> Result<i32> {
        let value = self.read(n)?;
        if n > 0 && (value >> (n - 1)) != 0 {
            Ok((value as i32) - (1i32 << n))
        } else {
            Ok(value as i32)
        }
    }

    #[inline]
    pub fn skip(&mut self, n: usize) -> Result<()> {
        if self.pos + n > self.end {
            return fail("skip past the end of the field");
        }
        self.pos += n;
        Ok(())
    }

    /// TS 102 366 clause H.2.1.2.1：每组 n 位加 1 位续传标记。
    pub fn variable_bits(&mut self, n: u32) -> Result<u32> {
        let mut value = 0u32;
        loop {
            value += self.read(n)?;
            if !self.flag()? {
                return Ok(value);
            }
            value = (value << n) + (1u32 << n);
        }
    }
}

#[derive(Clone, Copy)]
struct TableEntry {
    leaf: bool,
    value: i32,
    used: u8,
}

pub struct Huffman {
    table: Vec<TableEntry>,
    tree: Vec<[i32; 2]>,
    width: u32,
}

impl Huffman {
    pub fn from_codebook(cb: &Codebook) -> Self {
        let mut nodes: Vec<[Option<i32>; 2]> = vec![[None, None]];
        for (value, (&length, &code)) in cb.lengths.iter().zip(cb.codewords.iter()).enumerate() {
            let mut node = 0usize;
            for i in (0..length).rev() {
                let bit = ((code >> i) & 1) as usize;
                if i == 0 {
                    nodes[node][bit] = Some(-(value as i32) - 1);
                } else if let Some(child) = nodes[node][bit] {
                    node = child as usize;
                } else {
                    let child = nodes.len() as i32;
                    nodes.push([None, None]);
                    nodes[node][bit] = Some(child);
                    node = child as usize;
                }
            }
        }
        let tree: Vec<[i32; 2]> = nodes
            .into_iter()
            .map(|[l, r]| [l.unwrap_or(-1), r.unwrap_or(-1)])
            .collect();
        let width = 11u32;
        let table_len = 1usize << width;
        let mut table = Vec::with_capacity(table_len);
        for prefix in 0..table_len {
            let mut node = 0i32;
            let mut found = None;
            for used in 1..=width {
                let bit = (prefix >> (width - used)) & 1;
                node = tree[node as usize][bit];
                if node <= 0 {
                    found = Some(TableEntry { leaf: true, value: -node - 1, used: used as u8 });
                    break;
                }
            }
            table.push(found.unwrap_or(TableEntry { leaf: false, value: node, used: width as u8 }));
        }
        Self { table, tree, width }
    }

    #[inline]
    pub fn decode(&self, bits: &mut BitReader<'_>) -> Result<i32> {
        let entry = self.table[bits.peek(self.width) as usize];
        bits.skip(usize::from(entry.used))?;
        if entry.leaf {
            return Ok(entry.value);
        }
        let mut node = entry.value;
        loop {
            let bit = bits.read(1)? as usize;
            node = self.tree[node as usize][bit];
            if node <= 0 {
                return Ok(-node - 1);
            }
        }
    }
}
