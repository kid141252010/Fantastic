//! 静态表（ETSI TS 103 190-1 / TS 103 190-2 / TS 103 420），由 SoundWeaver 自动导出。

#[derive(Clone, Copy)]
pub struct Codebook {
    pub lengths: &'static [u8],
    pub codewords: &'static [u32],
}

pub static ACPL_HCB_ALPHA_COARSE_DF: Codebook = Codebook {
    lengths: &[
    15, 18, 17, 17, 16, 15, 15, 13, 12, 11, 10, 9, 8, 7, 4, 3, 1, 2, 5, 7, 8, 9, 10, 11,
    12, 13, 15, 16, 16, 17, 16, 18, 15,
    ],
    codewords: &[
    0x7c76, 0x3e3fe, 0x1f1f6, 0x1f1f7, 0xf8ea, 0x7c74, 0x7c7c, 0x1f1c, 0xf9e, 0x7ce, 0x3e2, 0x1f0,
    0xfa, 0x7e, 0xe, 0x6, 0x0, 0x2, 0x1e, 0x7f, 0xfb, 0x1f2, 0x3e6, 0x7c6,
    0xf9f, 0x1f1e, 0x7c7e, 0xf8fe, 0xf8fa, 0x1f1fe, 0xf8eb, 0x3e3ff, 0x7c77,
    ],
};

pub static ACPL_HCB_ALPHA_COARSE_DT: Codebook = Codebook {
    lengths: &[
    14, 16, 15, 16, 15, 15, 14, 13, 12, 12, 10, 9, 8, 7, 5, 3, 1, 2, 4, 7, 8, 9, 10, 11,
    12, 13, 14, 15, 15, 16, 15, 16, 14,
    ],
    codewords: &[
    0x3efc, 0xfbfa, 0x7ddc, 0xfbfe, 0x7dde, 0x7dfc, 0x3ef6, 0x1f76, 0xfba, 0xfbe, 0x3ec, 0x1f2,
    0xf8, 0x7e, 0x1e, 0x6, 0x0, 0x2, 0xe, 0x7f, 0xfa, 0x1f3, 0x3ed, 0x7dc,
    0xfbc, 0x1f7a, 0x3ef7, 0x7dfe, 0x7ddf, 0xfbff, 0x7ddd, 0xfbfb, 0x3efd,
    ],
};

pub static ACPL_HCB_ALPHA_COARSE_F0: Codebook = Codebook {
    lengths: &[
    10, 10, 9, 8, 6, 6, 5, 2, 1, 3, 5, 7, 7, 8, 9, 10, 10,
    ],
    codewords: &[
    0x3be, 0x3fe, 0x1fe, 0xfe, 0x3e, 0x3a, 0x1e, 0x2, 0x0, 0x6, 0x1c, 0x7e,
    0x76, 0xee, 0x1de, 0x3ff, 0x3bf,
    ],
};

pub static ACPL_HCB_ALPHA_FINE_DF: Codebook = Codebook {
    lengths: &[
    13, 17, 17, 17, 16, 17, 17, 17, 17, 16, 16, 16, 15, 15, 14, 13, 13, 12, 12, 11, 11, 11, 10, 10,
    10, 9, 8, 7, 7, 5, 4, 3, 1, 3, 4, 5, 6, 7, 8, 9, 9, 10, 10, 11, 11, 12, 12, 12,
    13, 13, 14, 15, 15, 16, 16, 17, 16, 16, 17, 16, 16, 17, 17, 17, 13,
    ],
    codewords: &[
    0x11de, 0x11ffe, 0x13dea, 0x13df6, 0x8eea, 0x13df7, 0x13dee, 0x13deb, 0x13dec, 0x8eee, 0x8ffe, 0x9efe,
    0x47fe, 0x4f7c, 0x23fe, 0x11fe, 0x13fe, 0x8f6, 0x9ee, 0x476, 0x47a, 0x4f6, 0x23a, 0x27a,
    0x27e, 0x13e, 0x9a, 0x4c, 0x4e, 0x12, 0xa, 0x6, 0x0, 0x7, 0xb, 0x10,
    0x22, 0x46, 0x9b, 0x13c, 0x11c, 0x23e, 0x23c, 0x4fe, 0x47e, 0x9fe, 0x8fe, 0x8f7,
    0x13ff, 0x11df, 0x27bc, 0x4f7e, 0x4776, 0x9efa, 0x9ef4, 0x13dfe, 0x8eeb, 0x8ee8, 0x13dff, 0x8ee9,
    0x8eef, 0x11fff, 0x13ded, 0x13def, 0x11dc,
    ],
};

pub static ACPL_HCB_ALPHA_FINE_DT: Codebook = Codebook {
    lengths: &[
    16, 18, 18, 18, 17, 17, 17, 18, 17, 17, 17, 16, 16, 16, 15, 15, 14, 14, 13, 13, 13, 12, 11, 11,
    10, 10, 9, 9, 7, 6, 5, 3, 1, 2, 5, 6, 7, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13, 14,
    14, 15, 15, 16, 16, 16, 17, 17, 17, 17, 17, 18, 17, 18, 18, 18, 16,
    ],
    codewords: &[
    0xeeee, 0x3b3ee, 0x3b3f6, 0x3b3fc, 0x1d9bc, 0x1d9bd, 0x1d9b2, 0x3b3fe, 0x1d9be, 0x1d9f6, 0x1d9fc, 0xecda,
    0xecfa, 0xeeef, 0x766e, 0x7776, 0x3b3a, 0x3bba, 0x1d9a, 0x1ddc, 0x1dde, 0xeec, 0x764, 0x772,
    0x3b0, 0x3b8, 0x1da, 0x1de, 0x72, 0x38, 0x1e, 0x6, 0x0, 0x2, 0x1f, 0x3a,
    0x73, 0x1df, 0x1db, 0x3ba, 0x3b1, 0x773, 0x765, 0xeed, 0xecc, 0x1d9e, 0x1d9c, 0x3bbe,
    0x3b3b, 0x777e, 0x767c, 0xeefe, 0xecfc, 0xecd8, 0x1d9fd, 0x1d9fa, 0x1d9bf, 0x1d9b6, 0x1d9b3, 0x3b3fd,
    0x1d9b7, 0x3b3ff, 0x3b3ef, 0x3b3f7, 0xeeff,
    ],
};

pub static ACPL_HCB_ALPHA_FINE_F0: Codebook = Codebook {
    lengths: &[
    10, 12, 11, 11, 10, 10, 9, 8, 7, 7, 8, 7, 6, 6, 4, 3, 1, 3, 4, 6, 6, 7, 8, 8,
    9, 9, 10, 10, 10, 10, 11, 12, 10,
    ],
    codewords: &[
    0x2ce, 0xb5e, 0x4fe, 0x5ae, 0x27e, 0x2de, 0x16a, 0xb2, 0x4a, 0x4b, 0xb6, 0x4e,
    0x24, 0x2e, 0xa, 0x6, 0x0, 0x7, 0x8, 0x2f, 0x26, 0x58, 0xb4, 0x9e,
    0x16e, 0x166, 0x2df, 0x2cf, 0x27c, 0x27d, 0x4ff, 0xb5f, 0x2d6,
    ],
};

pub static ACPL_HCB_BETA3_COARSE_DF: Codebook = Codebook {
    lengths: &[
    13, 12, 12, 11, 9, 6, 4, 2, 1, 3, 5, 7, 9, 11, 12, 13, 9,
    ],
    codewords: &[
    0xa93, 0x548, 0x54b, 0x2a7, 0xab, 0x14, 0x4, 0x0, 0x1, 0x3, 0xb, 0x2b,
    0xaa, 0x2a6, 0x54a, 0xa92, 0xa8,
    ],
};

pub static ACPL_HCB_BETA3_COARSE_DT: Codebook = Codebook {
    lengths: &[
    15, 15, 14, 12, 10, 7, 5, 3, 1, 2, 4, 6, 8, 11, 14, 14, 9,
    ],
    codewords: &[
    0xadc, 0xadd, 0x56c, 0x15a, 0x57, 0xb, 0x3, 0x1, 0x1, 0x1, 0x0, 0x4,
    0x14, 0xac, 0x56f, 0x56d, 0x2a,
    ],
};

pub static ACPL_HCB_BETA3_COARSE_F0: Codebook = Codebook {
    lengths: &[
    5, 3, 3, 2, 2, 3, 4, 6, 6,
    ],
    codewords: &[
    0x1, 0x6, 0x7, 0x1, 0x2, 0x1, 0x1, 0x1, 0x0,
    ],
};

pub static ACPL_HCB_BETA3_FINE_DF: Codebook = Codebook {
    lengths: &[
    14, 15, 14, 13, 13, 12, 11, 11, 9, 8, 7, 6, 5, 4, 3, 2, 2, 3, 4, 5, 6, 7, 8, 9,
    10, 11, 12, 12, 13, 14, 14, 14, 15,
    ],
    codewords: &[
    0x19e9, 0x33f7, 0x19f3, 0xcf5, 0xcfc, 0x67d, 0x33c, 0x7ff, 0xce, 0x66, 0x32, 0x18,
    0xd, 0x7, 0x2, 0x0, 0x2, 0x6, 0xe, 0x1e, 0x3e, 0x7e, 0xfe, 0x1fe,
    0x3fe, 0x7fe, 0x67f, 0x67b, 0xcf8, 0x19fa, 0x19f2, 0x19e8, 0x33f6,
    ],
};

pub static ACPL_HCB_BETA3_FINE_DT: Codebook = Codebook {
    lengths: &[
    16, 16, 16, 16, 16, 16, 15, 14, 12, 11, 10, 9, 8, 7, 5, 3, 1, 2, 4, 7, 8, 9, 10, 11,
    12, 13, 14, 15, 15, 16, 16, 16, 16,
    ],
    codewords: &[
    0x501e, 0x501d, 0x501c, 0x501b, 0x510e, 0x510d, 0x2809, 0x1442, 0x500, 0x281, 0x141, 0xa1,
    0x52, 0x2a, 0xb, 0x3, 0x1, 0x0, 0x4, 0x2b, 0x53, 0xa3, 0x145, 0x289,
    0x511, 0xa20, 0x1405, 0x280c, 0x2808, 0x510f, 0x510c, 0x501f, 0x501a,
    ],
};

pub static ACPL_HCB_BETA3_FINE_F0: Codebook = Codebook {
    lengths: &[
    7, 5, 4, 4, 4, 3, 3, 3, 3, 3, 4, 5, 6, 6, 7, 7, 7,
    ],
    codewords: &[
    0xd, 0x2, 0x0, 0xc, 0xe, 0x1, 0x3, 0x5, 0x4, 0x2, 0xd, 0x1f,
    0x3d, 0x7, 0x78, 0xc, 0x79,
    ],
};

pub static ACPL_HCB_BETA_COARSE_DF: Codebook = Codebook {
    lengths: &[
    8, 6, 4, 3, 1, 2, 5, 7, 8,
    ],
    codewords: &[
    0xfe, 0x3e, 0xe, 0x6, 0x0, 0x2, 0x1e, 0x7e, 0xff,
    ],
};

pub static ACPL_HCB_BETA_COARSE_DT: Codebook = Codebook {
    lengths: &[
    8, 7, 5, 3, 1, 2, 4, 6, 8,
    ],
    codewords: &[
    0xfe, 0x7e, 0x1e, 0x6, 0x0, 0x2, 0xe, 0x3e, 0xff,
    ],
};

pub static ACPL_HCB_BETA_COARSE_F0: Codebook = Codebook {
    lengths: &[
    1, 2, 3, 4, 4,
    ],
    codewords: &[
    0x0, 0x2, 0x6, 0xe, 0xf,
    ],
};

pub static ACPL_HCB_BETA_FINE_DF: Codebook = Codebook {
    lengths: &[
    13, 12, 10, 9, 8, 7, 5, 3, 1, 2, 4, 7, 8, 9, 9, 11, 13,
    ],
    codewords: &[
    0x1f1e, 0xf8e, 0x3e2, 0x1f2, 0xfa, 0x7e, 0x1e, 0x6, 0x0, 0x2, 0xe, 0x7f,
    0xfb, 0x1f3, 0x1f0, 0x7c6, 0x1f1f,
    ],
};

pub static ACPL_HCB_BETA_FINE_DT: Codebook = Codebook {
    lengths: &[
    15, 14, 12, 10, 8, 7, 5, 3, 1, 2, 4, 7, 7, 9, 11, 13, 15,
    ],
    codewords: &[
    0x7dfe, 0x3efe, 0xfbe, 0x3ee, 0xfa, 0x7e, 0x1e, 0x6, 0x0, 0x2, 0xe, 0x7f,
    0x7c, 0x1f6, 0x7de, 0x1f7e, 0x7dff,
    ],
};

pub static ACPL_HCB_BETA_FINE_F0: Codebook = Codebook {
    lengths: &[
    1, 2, 3, 4, 5, 6, 7, 8, 8,
    ],
    codewords: &[
    0x0, 0x2, 0x6, 0xe, 0x1e, 0x3e, 0x7e, 0xfe, 0xff,
    ],
};

pub static ACPL_HCB_GAMMA_COARSE_DF: Codebook = Codebook {
    lengths: &[
    16, 16, 16, 16, 16, 16, 16, 16, 16, 15, 15, 14, 13, 13, 11, 10, 8, 7, 4, 2, 1, 3, 5, 7,
    8, 10, 11, 13, 13, 14, 15, 16, 16, 16, 16, 16, 16, 16, 16, 16, 8,
    ],
    codewords: &[
    0x53e1, 0x53e0, 0x53db, 0x53da, 0x53d9, 0x53e2, 0x53e4, 0x53ea, 0x53eb, 0x29ea, 0x29f4, 0x14f4,
    0xa78, 0xa7f, 0x299, 0x14d, 0x51, 0x2a, 0x4, 0x0, 0x1, 0x3, 0xb, 0x2b,
    0x52, 0x14e, 0x298, 0xa7e, 0xa79, 0x14f7, 0x29f6, 0x53ef, 0x53ee, 0x53e7, 0x53e6, 0x53e3,
    0x53e5, 0x53d8, 0x53d7, 0x53d6, 0x50,
    ],
};

pub static ACPL_HCB_GAMMA_COARSE_DT: Codebook = Codebook {
    lengths: &[
    17, 17, 17, 17, 16, 17, 16, 16, 16, 15, 14, 13, 12, 12, 10, 9, 8, 7, 5, 3, 1, 2, 4, 7,
    8, 10, 11, 12, 13, 13, 14, 15, 16, 16, 16, 17, 17, 17, 17, 17, 9,
    ],
    codewords: &[
    0xa7f3, 0xa7f1, 0xa7f9, 0xa7f8, 0x50e1, 0xa7fe, 0x50e8, 0x50eb, 0x53fe, 0x29fd, 0x143b, 0xa1b,
    0x50c, 0x53e, 0x142, 0xa0, 0x52, 0x2b, 0xb, 0x3, 0x1, 0x0, 0x4, 0x2a,
    0x51, 0x14e, 0x29e, 0x50f, 0xa7e, 0xa1a, 0x1439, 0x2871, 0x50ea, 0x50e9, 0x50e0, 0xa7ff,
    0xa7fb, 0xa7fa, 0xa7f2, 0xa7f0, 0xa6,
    ],
};

pub static ACPL_HCB_GAMMA_COARSE_F0: Codebook = Codebook {
    lengths: &[
    13, 13, 13, 13, 11, 9, 7, 6, 5, 3, 2, 3, 3, 4, 3, 3, 8, 11, 12, 13, 13,
    ],
    codewords: &[
    0xaf4, 0xaf8, 0xaf9, 0xafb, 0x2bc, 0xae, 0x2a, 0x14, 0xb, 0x1, 0x3, 0x5,
    0x0, 0x4, 0x4, 0x3, 0x56, 0x2bf, 0x57b, 0xaf5, 0xafa,
    ],
};

pub static ACPL_HCB_GAMMA_FINE_DF: Codebook = Codebook {
    lengths: &[
    17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 16, 16, 16, 15, 15, 15,
    14, 14, 13, 13, 13, 12, 11, 11, 10, 9, 8, 7, 6, 5, 4, 3, 1, 3, 4, 5, 6, 7, 9, 9,
    10, 11, 11, 12, 13, 13, 14, 14, 14, 15, 15, 15, 16, 16, 16, 17, 17, 17, 17, 17, 17, 17, 17, 17,
    17, 17, 17, 17, 17, 17, 17, 17, 17,
    ],
    codewords: &[
    0x13e1f, 0x13e35, 0x13e1e, 0x13e1d, 0x13e1c, 0x13e1b, 0x13e1a, 0x13e19, 0x13e34, 0x13e33, 0x13e18, 0x13ec2,
    0x13ec1, 0x13ece, 0x13edf, 0x13e17, 0x13ede, 0x13edd, 0x9d52, 0x9f18, 0x9f1b, 0x4eaa, 0x4ea8, 0x4fb1,
    0x2753, 0x2757, 0x13a8, 0x13e0, 0x13ee, 0x9d6, 0x4e9, 0x4fa, 0x27b, 0x13c, 0x9c, 0x4d,
    0x21, 0x12, 0xb, 0x7, 0x0, 0x6, 0xa, 0x11, 0x20, 0x4c, 0x13f, 0x13b,
    0x27a, 0x4f9, 0x4e8, 0x9d7, 0x13ef, 0x13e2, 0x27da, 0x27c7, 0x2752, 0x4fb6, 0x4eac, 0x4eab,
    0x9f65, 0x9d5a, 0x9d53, 0x13ecd, 0x13edc, 0x13ecc, 0x13ecf, 0x13ec9, 0x13e32, 0x13ec3, 0x13e16, 0x13ec0,
    0x13ec8, 0x13e15, 0x13e14, 0x13e13, 0x13e12, 0x13e11, 0x13e10, 0x13ab7, 0x13ab6,
    ],
};

pub static ACPL_HCB_GAMMA_FINE_DT: Codebook = Codebook {
    lengths: &[
    18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 17, 17, 17, 17, 17, 16, 16, 16, 15,
    15, 15, 14, 14, 13, 13, 12, 12, 11, 10, 9, 8, 7, 6, 5, 2, 1, 3, 5, 6, 7, 8, 10, 10,
    11, 12, 13, 13, 14, 14, 15, 15, 15, 15, 16, 16, 17, 17, 17, 17, 17, 17, 18, 18, 18, 18, 18, 18,
    18, 18, 18, 18, 18, 18, 18, 18, 18,
    ],
    codewords: &[
    0x31e44, 0x31d1d, 0x31e42, 0x31e16, 0x31e41, 0x31e47, 0x31d1c, 0x31e43, 0x31e73, 0x31e72, 0x31e15, 0x31e70,
    0x31e75, 0x31e7f, 0x31e7e, 0x18e88, 0x18d8b, 0x18e8f, 0x18f0e, 0x18f3e, 0xc746, 0xc796, 0xc79e, 0x6361,
    0x63c9, 0x63d8, 0x31d0, 0x31e6, 0x18d9, 0x18f1, 0xc6d, 0xc7a, 0x63b, 0x31c, 0x18c, 0xc1,
    0x62, 0x33, 0x1b, 0x2, 0x0, 0x7, 0x1a, 0x32, 0x61, 0xc0, 0x31f, 0x31a,
    0x637, 0xc75, 0x18f7, 0x18e9, 0x31ed, 0x31e0, 0x63d9, 0x63ca, 0x6363, 0x6360, 0xc786, 0xc745,
    0x18f3b, 0x18f2e, 0x18e89, 0x18d88, 0x18d8a, 0x18d89, 0x31e5f, 0x31e74, 0x31e40, 0x31e71, 0x31e46, 0x31e5e,
    0x31e1f, 0x31e45, 0x31e1e, 0x31e14, 0x31e17, 0x31e13, 0x31e12, 0x31e11, 0x31e10,
    ],
};

pub static ACPL_HCB_GAMMA_FINE_F0: Codebook = Codebook {
    lengths: &[
    12, 13, 13, 12, 12, 12, 12, 11, 9, 10, 9, 8, 8, 7, 7, 6, 5, 5, 4, 4, 3, 3, 4, 4,
    5, 5, 5, 5, 4, 3, 4, 7, 8, 9, 10, 11, 11, 12, 12, 12, 12,
    ],
    codewords: &[
    0x4b6, 0x1c6d, 0x1c6c, 0x49b, 0x4b5, 0x4b7, 0xe35, 0x24e, 0x1c7, 0x38c, 0x97, 0x48,
    0xe2, 0x70, 0x73, 0x13, 0x8, 0x17, 0x5, 0xc, 0x4, 0x1, 0xd, 0xa,
    0x1f, 0x1e, 0x16, 0x1d, 0x6, 0x0, 0x7, 0x72, 0x4a, 0x92, 0x12c, 0x24f,
    0x24c, 0xe34, 0x4b4, 0x49a, 0xe37,
    ],
};

pub static AJCC_HCB_DRY_COARSE_DF: Codebook = Codebook {
    lengths: &[
    19, 17, 16, 14, 14, 13, 10, 8, 6, 4, 3, 1, 2, 5, 7, 9, 11, 13, 14, 16, 16, 18, 19,
    ],
    codewords: &[
    0x554fa, 0x1553f, 0xaa9e, 0x2aa6, 0x2aa4, 0x1550, 0x2ab, 0xab, 0x2b, 0xb, 0x4, 0x0,
    0x3, 0x14, 0x54, 0x154, 0x555, 0x1551, 0x2aa5, 0xaa9c, 0xaa9d, 0x2aa7c, 0x554fb,
    ],
};

pub static AJCC_HCB_DRY_COARSE_DT: Codebook = Codebook {
    lengths: &[
    16, 17, 17, 15, 14, 13, 11, 8, 7, 5, 2, 1, 3, 4, 6, 9, 10, 13, 14, 15, 17, 16, 17,
    ],
    codewords: &[
    0xaaa7, 0x1554c, 0x15575, 0x555c, 0x2aaf, 0x1556, 0x554, 0xab, 0x54, 0x14, 0x3, 0x0,
    0x4, 0xb, 0x2b, 0x154, 0x2ab, 0x1555, 0x2aa8, 0x5552, 0x1554d, 0xaabb, 0x15574,
    ],
};

pub static AJCC_HCB_DRY_COARSE_F0: Codebook = Codebook {
    lengths: &[
    9, 7, 6, 2, 2, 2, 3, 4, 5, 8, 10, 10,
    ],
    codewords: &[
    0x33, 0xd, 0x7, 0x1, 0x3, 0x2, 0x1, 0x0, 0x2, 0x18, 0x64, 0x65,
    ],
};

pub static AJCC_HCB_DRY_FINE_DF: Codebook = Codebook {
    lengths: &[
    20, 20, 19, 18, 17, 17, 16, 15, 15, 14, 13, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 1, 3,
    4, 5, 6, 7, 8, 9, 10, 11, 13, 14, 14, 15, 16, 16, 18, 18, 17, 18, 19, 20, 20,
    ],
    codewords: &[
    0x6015a, 0x6015b, 0x30039, 0x18057, 0xc4a8, 0xc02a, 0x6006, 0x312b, 0x300b, 0x1894, 0xc4b, 0xc03,
    0x624, 0x301, 0x181, 0xc1, 0x61, 0x38, 0x19, 0xd, 0x5, 0x1, 0x1, 0x0,
    0x4, 0xf, 0x1d, 0x39, 0x63, 0xc5, 0x188, 0x313, 0xc01, 0x1800, 0x1804, 0x3002,
    0x6014, 0x6255, 0x1801d, 0x18952, 0xc00f, 0x18953, 0x30038, 0x60158, 0x60159,
    ],
};

pub static AJCC_HCB_DRY_FINE_DT: Codebook = Codebook {
    lengths: &[
    19, 20, 19, 21, 20, 19, 19, 18, 18, 17, 17, 16, 13, 12, 11, 10, 9, 8, 7, 6, 5, 2, 1, 3,
    5, 6, 7, 8, 9, 10, 11, 12, 12, 15, 16, 17, 18, 18, 18, 19, 19, 20, 19, 21, 19,
    ],
    codewords: &[
    0x58042, 0xb00c7, 0x5805e, 0x16018c, 0xb0092, 0x58043, 0x5805f, 0x2c025, 0x2c026, 0x16019, 0x16011, 0xb00a,
    0x1600, 0xb01, 0x582, 0x2c2, 0x163, 0xb3, 0x5f, 0x2d, 0x14, 0x3, 0x0, 0x4,
    0x15, 0x2e, 0x5e, 0xb2, 0x162, 0x2c3, 0x583, 0xb02, 0xb03, 0x5807, 0xb00d, 0x16016,
    0x2c020, 0x2c030, 0x2c027, 0x58048, 0x5805c, 0xb0093, 0x5805d, 0x16018d, 0x58062,
    ],
};

pub static AJCC_HCB_DRY_FINE_F0: Codebook = Codebook {
    lengths: &[
    10, 11, 9, 8, 8, 7, 3, 4, 3, 3, 3, 3, 4, 4, 4, 5, 4, 7, 9, 10, 12, 12, 10,
    ],
    codewords: &[
    0x168, 0x2d2, 0xb5, 0x5f, 0x5b, 0x2c, 0x0, 0x2, 0x7, 0x3, 0x5, 0x4,
    0x3, 0x4, 0xc, 0xa, 0xd, 0x2e, 0xbd, 0x178, 0x5a6, 0x5a7, 0x179,
    ],
};

pub static AJCC_HCB_WET_COARSE_DF: Codebook = Codebook {
    lengths: &[
    18, 19, 19, 19, 19, 19, 19, 19, 19, 17, 14, 12, 11, 10, 10, 9, 8, 7, 4, 2, 1, 3, 5, 7,
    8, 10, 10, 11, 12, 12, 13, 19, 19, 18, 18, 18, 18, 18, 18, 18, 18,
    ],
    codewords: &[
    0x2bdd5, 0x57ba8, 0x57ba9, 0x57ba2, 0x57ba3, 0x57ba0, 0x57ba1, 0x57ba6, 0x57ba7, 0x15eeb, 0x2bdc, 0xac5,
    0x57a, 0x2bf, 0x2bc, 0x159, 0xad, 0x54, 0xb, 0x3, 0x0, 0x4, 0x14, 0x55,
    0xae, 0x2b0, 0x2be, 0x563, 0xac4, 0xaf6, 0x15ef, 0x57ba4, 0x57ba5, 0x2bdda, 0x2bddb, 0x2bdd8,
    0x2bdd9, 0x2bdde, 0x2bddf, 0x2bddc, 0x2bddd,
    ],
};

pub static AJCC_HCB_WET_COARSE_DT: Codebook = Codebook {
    lengths: &[
    17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 15, 14, 13, 13, 11, 10, 9, 6, 4, 3, 1, 2, 5, 7,
    9, 10, 11, 12, 13, 14, 14, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17,
    ],
    codewords: &[
    0x157ea, 0x157eb, 0x157e8, 0x157e9, 0x157ee, 0x157ef, 0x157ec, 0x157ed, 0x157e2, 0x157e3, 0x55fd, 0x2ae2,
    0x1572, 0x1570, 0x55e, 0x2ad, 0x154, 0x2b, 0xb, 0x4, 0x0, 0x3, 0x14, 0x54,
    0x155, 0x2ac, 0x55d, 0xabe, 0x1573, 0x2ae3, 0x2aff, 0x157e0, 0x157e1, 0x157e6, 0x157e7, 0x157e4,
    0x157e5, 0x157f2, 0x157f3, 0x157f0, 0x157f1,
    ],
};

pub static AJCC_HCB_WET_COARSE_F0: Codebook = Codebook {
    lengths: &[
    12, 16, 16, 15, 14, 13, 12, 10, 6, 4, 1, 3, 3, 3, 5, 7, 8, 10, 11, 12, 11,
    ],
    codewords: &[
    0x2af, 0x2ae4, 0x2ae5, 0x1573, 0xab8, 0x55d, 0x2aa, 0xa9, 0xb, 0x3, 0x1, 0x0,
    0x2, 0x3, 0x4, 0x14, 0x2b, 0xa8, 0x156, 0x2ab, 0x154,
    ],
};

pub static AJCC_HCB_WET_FINE_DF: Codebook = Codebook {
    lengths: &[
    19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 18, 19, 16, 14, 14, 13,
    13, 12, 12, 12, 12, 11, 11, 10, 10, 9, 8, 7, 6, 5, 4, 3, 1, 3, 4, 5, 6, 8, 8, 9,
    10, 11, 11, 12, 12, 12, 12, 13, 13, 13, 14, 14, 15, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19,
    19, 19, 19, 19, 19, 19, 19, 20, 20,
    ],
    codewords: &[
    0x715d6, 0x715d7, 0x715d4, 0x715d5, 0x7ff2a, 0x7ff2b, 0x7ff28, 0x7ff29, 0x7ff2e, 0x7ff2f, 0x7ff2c, 0x7ff2d,
    0x7ff22, 0x7ff23, 0x7ff20, 0x7ff21, 0x7ff26, 0x7ff27, 0x38ae9, 0x715d0, 0xe2bb, 0x38af, 0x386c, 0x1c54,
    0x1c37, 0xffd, 0xe29, 0xe1a, 0xe19, 0x7fd, 0x709, 0x38b, 0x385, 0x1c4, 0xe3, 0x7e,
    0x39, 0x1d, 0xc, 0x4, 0x0, 0x5, 0xd, 0x1e, 0x3e, 0xe0, 0xfe, 0x1fe,
    0x387, 0x708, 0x7fc, 0xe18, 0xe28, 0xffc, 0xfff, 0x1c55, 0x1c56, 0x1ffd, 0x386d, 0x3ff8,
    0x715c, 0x7ff24, 0x7ff25, 0x7ff3a, 0x7ff3b, 0x7ff38, 0x7ff39, 0x7ff3e, 0x7ff3f, 0x7ff3c, 0x7ff3d, 0x7ff32,
    0x7ff33, 0x7ff30, 0x7ff31, 0x7ff36, 0x7ff37, 0x7ff34, 0x7ff35, 0xe2ba2, 0xe2ba3,
    ],
};

pub static AJCC_HCB_WET_FINE_DT: Codebook = Codebook {
    lengths: &[
    19, 20, 20, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 18, 16, 16,
    16, 16, 15, 15, 14, 13, 13, 12, 11, 10, 9, 8, 7, 6, 5, 3, 1, 2, 5, 6, 7, 8, 9, 11,
    11, 12, 13, 13, 14, 14, 15, 16, 15, 17, 17, 18, 17, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19,
    19, 19, 19, 19, 19, 19, 19, 19, 19,
    ],
    codewords: &[
    0x5e213, 0xbc424, 0xbc425, 0x5e7a2, 0x5e7a3, 0x5e7a0, 0x5e7a1, 0x5e7a6, 0x5e7a7, 0x5e7a4, 0x5e7a5, 0x5e74a,
    0x5e74b, 0x5e748, 0x5e749, 0x5e74e, 0x5e74f, 0x5e74c, 0x5e74d, 0x5e742, 0x5e743, 0x2f3aa, 0xbcf5, 0xbc43,
    0xbc47, 0xbc41, 0x5e26, 0x5e22, 0x2f3b, 0x179c, 0x1784, 0xbc3, 0x5e3, 0x2f2, 0x17e, 0xbe,
    0x5c, 0x2c, 0x14, 0x4, 0x0, 0x3, 0x15, 0x2d, 0x5d, 0xbd, 0x17f, 0x5e0,
    0x5e6, 0xbc5, 0x1785, 0x179f, 0x2f12, 0x2f3c, 0x5e27, 0xbc46, 0x5e7b, 0x17880, 0x17885, 0x2f108,
    0x17881, 0x5e740, 0x5e741, 0x5e746, 0x5e747, 0x5e744, 0x5e745, 0x5e75a, 0x5e75b, 0x5e758, 0x5e759, 0x5e75e,
    0x5e75f, 0x5e75c, 0x5e75d, 0x5e752, 0x5e753, 0x5e750, 0x5e751, 0x5e756, 0x5e757,
    ],
};

pub static AJCC_HCB_WET_FINE_F0: Codebook = Codebook {
    lengths: &[
    12, 15, 16, 17, 17, 15, 15, 15, 14, 15, 13, 13, 13, 11, 10, 9, 7, 6, 5, 4, 2, 3, 3, 4,
    4, 4, 4, 4, 5, 6, 7, 8, 9, 9, 10, 11, 11, 12, 12, 13, 10,
    ],
    codewords: &[
    0xe37, 0x712a, 0xe359, 0x1c6b0, 0x1c6b1, 0x71ad, 0x712b, 0x7128, 0x38d7, 0x7129, 0x1c6a, 0x1c48,
    0x1c49, 0x71f, 0x38c, 0x108, 0x43, 0x20, 0x11, 0x6, 0x0, 0x5, 0x6, 0x4,
    0x5, 0x7, 0x9, 0xf, 0x1d, 0x39, 0x70, 0x85, 0x109, 0x1c5, 0x388, 0x713,
    0x71e, 0xe34, 0xe36, 0x1c4b, 0x38e,
    ],
};

pub static AJOC_HCB_DRY_COARSE_DF: Codebook = Codebook {
    lengths: &[
    1, 2, 5, 6, 7, 7, 9, 10, 11, 12, 12, 13, 13, 14, 14, 15, 15, 16, 16, 17, 17, 18, 18, 18,
    19, 19, 19, 18, 19, 18, 18, 17, 17, 16, 16, 15, 15, 14, 14, 13, 13, 12, 12, 11, 10, 10, 8, 8,
    6, 5, 3,
    ],
    codewords: &[
    0x0, 0x3, 0x14, 0x2d, 0x59, 0x5c, 0x174, 0x2c7, 0x58b, 0xb14, 0xba8, 0x162a,
    0x1752, 0x2c56, 0x2ea7, 0x58ce, 0x5d4d, 0xb15d, 0xb19f, 0x162bc, 0x1756b, 0x2c573, 0x2c571, 0x2c570,
    0x58ae4, 0x58ae5, 0x5d5a9, 0x2ead5, 0x5d5a8, 0x2c57a, 0x2c57b, 0x1633c, 0x1633d, 0xbab4, 0xb15f, 0x5d5b,
    0x5d4c, 0x2eac, 0x2c66, 0x1757, 0x1632, 0xbaa, 0xb18, 0x58d, 0x2eb, 0x2c4, 0xbb, 0xb0,
    0x2f, 0x15, 0x4,
    ],
};

pub static AJOC_HCB_DRY_COARSE_DT: Codebook = Codebook {
    lengths: &[
    27, 27, 27, 21, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 21, 21, 21, 27, 21, 27, 20, 20, 21,
    20, 17, 19, 18, 17, 17, 16, 17, 16, 15, 15, 15, 14, 14, 14, 13, 12, 12, 11, 10, 10, 8, 7, 6,
    5, 3, 1, 2, 5, 7, 7, 8, 9, 10, 11, 12, 12, 13, 13, 14, 14, 15, 15, 16, 16, 17, 17, 16,
    18, 18, 18, 18, 7, 21, 20, 21, 27, 27, 27, 27, 20, 21, 27, 27, 27, 27, 22, 27, 27, 27, 27, 28,
    28, 28, 28, 27, 27,
    ],
    codewords: &[
    0x52447a6, 0x52447a7, 0x52447a4, 0x14911f, 0x52447a5, 0x52447ba, 0x52447bb, 0x52447b8, 0x52447b9, 0x52447be, 0x52447bf, 0x52447bc,
    0x52447bd, 0x52447b2, 0x52447b3, 0x14913c, 0x14913d, 0x14913a, 0x52447b0, 0x14913b, 0x52447b1, 0xa489f, 0xa4881, 0x149100,
    0xa4883, 0x14917, 0x52446, 0x29011, 0x14912, 0x14819, 0xa4f6, 0x14809, 0xa48a, 0x527a, 0x5241, 0x5207,
    0x2923, 0x2921, 0x2900, 0x1487, 0xa4e, 0xa41, 0x525, 0x299, 0x291, 0xa7, 0x5c, 0x2f,
    0x15, 0x4, 0x0, 0x3, 0x16, 0x50, 0x5d, 0xa5, 0x14d, 0x298, 0x526, 0xa42,
    0xa49, 0x1486, 0x149f, 0x2902, 0x293c, 0x5203, 0x5240, 0xa405, 0xa40d, 0x14818, 0x14916, 0xa4f7,
    0x29010, 0x29222, 0x29226, 0x29221, 0x51, 0x149101, 0xa4882, 0x149138, 0x52447b6, 0x52447b7, 0x52447b4, 0x52447b5,
    0xa488e, 0x149139, 0x52447aa, 0x52447ab, 0x52447a8, 0x52447a9, 0x29223c, 0x52447ae, 0x52447af, 0x52447ac, 0x52447ad, 0xa488f42,
    0xa488f43, 0xa488f40, 0xa488f41, 0x52447a2, 0x52447a3,
    ],
};

pub static AJOC_HCB_DRY_COARSE_F0: Codebook = Codebook {
    lengths: &[
    17, 17, 18, 17, 17, 16, 16, 15, 14, 15, 14, 13, 13, 12, 12, 11, 11, 11, 10, 10, 9, 8, 7, 5,
    3, 1, 3, 4, 5, 5, 4, 7, 9, 10, 11, 11, 11, 12, 12, 13, 13, 14, 14, 15, 16, 17, 16, 16,
    16, 18, 16,
    ],
    codewords: &[
    0x15b63, 0x15a3b, 0x2b474, 0x15a3c, 0x15a3d, 0xadbc, 0xad1f, 0x56df, 0x2b6d, 0x5688, 0x2b45, 0x15b5,
    0x151a, 0xad3, 0xad0, 0x56f, 0x56c, 0x541, 0x2b5, 0x2a2, 0x159, 0xa9, 0x55, 0x17,
    0x6, 0x0, 0x4, 0xe, 0x14, 0x16, 0xf, 0x57, 0x158, 0x2a1, 0x540, 0x547,
    0x56e, 0xa8c, 0xad2, 0x151b, 0x15b4, 0x2b46, 0x2b6e, 0x5689, 0xad1c, 0x15b62, 0xadbd, 0xadb0,
    0xadb2, 0x2b475, 0xadb3,
    ],
};

pub static AJOC_HCB_DRY_FINE_DF: Codebook = Codebook {
    lengths: &[
    1, 3, 4, 5, 6, 7, 8, 8, 8, 9, 9, 10, 11, 11, 12, 12, 13, 13, 13, 13, 14, 14, 14, 14,
    15, 15, 15, 16, 16, 16, 16, 17, 17, 18, 17, 18, 18, 18, 18, 20, 19, 19, 19, 20, 20, 20, 19, 19,
    21, 21, 19, 20, 21, 21, 20, 20, 21, 19, 19, 21, 19, 18, 19, 19, 18, 18, 18, 17, 18, 17, 17, 17,
    16, 16, 16, 16, 15, 15, 15, 14, 14, 14, 14, 14, 13, 13, 12, 12, 12, 11, 11, 10, 10, 9, 9, 8,
    7, 6, 6, 4, 3,
    ],
    codewords: &[
    0x0, 0x4, 0xd, 0x1f, 0x3c, 0x64, 0xc6, 0xcb, 0xcf, 0x18b, 0x188, 0x338,
    0x629, 0x673, 0xca1, 0xca7, 0x18a0, 0x1948, 0x19ca, 0x19df, 0x3281, 0x3299, 0x3393, 0x33bd,
    0x6524, 0x6720, 0x6778, 0xc50c, 0xce46, 0xca4e, 0xce4b, 0x19c8b, 0x19499, 0x31439, 0x19c8e, 0x32931,
    0x3143a, 0x32934, 0x3392a, 0xc50e0, 0x65261, 0x62871, 0x67253, 0xc50e1, 0xce454, 0xce455, 0x62868, 0x6722b,
    0x19de7c, 0x19de7d, 0x6779e, 0xcef3f, 0x18a1a6, 0x18a1a7, 0xca4d6, 0xca4d7, 0x18a1a4, 0x67250, 0x67251, 0x18a1a5,
    0x6526a, 0x3392b, 0x67252, 0x65260, 0x33bce, 0x33914, 0x31435, 0x19de6, 0x3143b, 0x1949b, 0x19c8f, 0x18a1b,
    0xcef2, 0xca4f, 0xce44, 0xc50f, 0x6724, 0x6721, 0x6525, 0x3396, 0x3397, 0x3298, 0x3280, 0x3142,
    0x194d, 0x1941, 0xcee, 0xca5, 0xc51, 0x676, 0x651, 0x33a, 0x315, 0x195, 0x189, 0xc7,
    0x66, 0x3d, 0x30, 0xe, 0x5,
    ],
};

pub static AJOC_HCB_DRY_FINE_DT: Codebook = Codebook {
    lengths: &[
    28, 29, 29, 29, 29, 29, 21, 29, 29, 29, 29, 29, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28,
    28, 28, 28, 28, 28, 28, 21, 28, 21, 21, 28, 28, 28, 28, 21, 28, 28, 28, 20, 21, 21, 20, 21, 21,
    20, 20, 17, 18, 20, 19, 18, 18, 17, 18, 18, 17, 17, 17, 17, 17, 17, 17, 16, 16, 16, 15, 15, 15,
    15, 15, 15, 14, 14, 14, 14, 13, 13, 13, 12, 12, 12, 12, 11, 11, 10, 10, 9, 9, 9, 8, 8, 7,
    6, 6, 4, 3, 1, 3, 4, 6, 6, 7, 8, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 12, 12, 13,
    13, 13, 14, 14, 14, 14, 14, 14, 15, 15, 15, 15, 15, 16, 16, 17, 16, 17, 17, 18, 18, 17, 17, 18,
    18, 18, 18, 18, 19, 19, 17, 7, 19, 28, 21, 20, 19, 28, 21, 28, 28, 28, 28, 28, 28, 28, 28, 21,
    28, 20, 21, 28, 28, 28, 28, 28, 28, 28, 28, 28, 22, 28, 28, 29, 29, 29, 29, 29, 29, 29, 29, 29,
    29, 29, 29, 29, 29, 29, 29, 28, 28,
    ],
    codewords: &[
    0xcbfcc4d, 0x197f9898, 0x197f9899, 0x197f9892, 0x197f9893, 0x197f9890, 0x197f99, 0x197f9891, 0x197f9896, 0x197f9897, 0x197f9894, 0x197f9895,
    0xcbfcc5a, 0xcbfcc5b, 0xcbfcc58, 0xcbfcc59, 0xcbfcc5e, 0xcbfcc5f, 0xcbfcc5c, 0xcbfcc5d, 0xcbfcc52, 0xcbfcc53, 0xcbfcc50, 0xcbfcc51,
    0xcbfcc56, 0xcbfcc57, 0xcbfcc54, 0xcbfcc55, 0xcbfcc6a, 0xcbfcc6b, 0x197ff4, 0xcbfcc68, 0x197ff5, 0x197f9e, 0xcbfcc69, 0xcbfcc6e,
    0xcbfcc6f, 0xcbfcc6c, 0x197f9f, 0xcbfcc6d, 0xcbfcc62, 0xcbfcc63, 0xcbffb, 0x197f9c, 0x197f9d, 0xfe85c, 0x197ff2, 0x197ff3,
    0xfe85d, 0xcbffe, 0x1fd46, 0x3fa8a, 0xcbfff, 0x654c6, 0x3fa10, 0x32ff0, 0x1fd47, 0x32ff1, 0x32a62, 0x195a1,
    0x197fe, 0x197f2, 0x1fd44, 0x1953e, 0x1953f, 0x19532, 0xcad1, 0xfeaa, 0xca9e, 0x79a8, 0x7f43, 0x79a9,
    0x654d, 0x6569, 0x654e, 0x3fa9, 0x3fa2, 0x32f8, 0x32a5, 0x1e6b, 0x1e68, 0x195b, 0xfeb, 0xf37,
    0xcab, 0xcaa, 0x7f1, 0x65e, 0x3fb, 0x3cc, 0x1e7, 0x1e4, 0x194, 0xfc, 0xc8, 0x67,
    0x3d, 0x31, 0xd, 0x5, 0x0, 0x4, 0xe, 0x30, 0x3e, 0x78, 0xc9, 0xfd,
    0xff, 0x196, 0x1e5, 0x32e, 0x3f9, 0x657, 0x7f0, 0xca8, 0xcac, 0xf36, 0xfe9, 0x197d,
    0x197e, 0x1e69, 0x32a4, 0x32b5, 0x32f9, 0x3fa3, 0x3cd5, 0x3fab, 0x65fd, 0x7f40, 0x7f41, 0x7f50,
    0x7f54, 0xcbfd, 0xcbf8, 0x19530, 0xcbfe, 0x195a0, 0x197f3, 0x32ff2, 0x32a67, 0x1fd09, 0x1fd0a, 0x32a66,
    0x3fa11, 0x3fa8b, 0x3fa16, 0x3faae, 0x654c7, 0x7f42f, 0x1fd56, 0x66, 0x7f55e, 0xcbfcc60, 0x197ff0, 0xcbffc,
    0x7f55f, 0xcbfcc61, 0x197ff1, 0xcbfcc66, 0xcbfcc67, 0xcbfcc64, 0xcbfcc65, 0xcbfcc7a, 0xcbfcc7b, 0xcbfcc78, 0xcbfcc79, 0x197f9a,
    0xcbfcc7e, 0xcbffd, 0x197f9b, 0xcbfcc7f, 0xcbfcc7c, 0xcbfcc7d, 0xcbfcc72, 0xcbfcc73, 0xcbfcc70, 0xcbfcc71, 0xcbfcc76, 0xcbfcc77,
    0x32ff30, 0xcbfcc74, 0xcbfcc75, 0x197f988a, 0x197f988b, 0x197f9888, 0x197f9889, 0x197f988e, 0x197f988f, 0x197f988c, 0x197f988d, 0x197f9882,
    0x197f9883, 0x197f9880, 0x197f9881, 0x197f9886, 0x197f9887, 0x197f9884, 0x197f9885, 0xcbfcc4e, 0xcbfcc4f,
    ],
};

pub static AJOC_HCB_DRY_FINE_F0: Codebook = Codebook {
    lengths: &[
    18, 21, 21, 18, 19, 21, 19, 18, 19, 19, 19, 19, 17, 18, 17, 16, 16, 17, 17, 16, 15, 15, 15, 15,
    14, 14, 14, 14, 13, 13, 13, 13, 13, 13, 12, 12, 12, 11, 11, 11, 11, 10, 9, 9, 8, 7, 7, 6,
    5, 3, 1, 3, 5, 5, 6, 6, 6, 7, 6, 6, 6, 8, 9, 10, 10, 11, 11, 12, 12, 12, 12, 13,
    13, 13, 13, 13, 14, 14, 15, 14, 15, 16, 15, 17, 16, 17, 17, 16, 17, 19, 19, 19, 18, 19, 19, 18,
    18, 19, 21, 19, 17,
    ],
    codewords: &[
    0x3505b, 0x1a82d6, 0x1a82d7, 0x228dc, 0x4512e, 0x1a82d4, 0x4512f, 0x228dd, 0x4512c, 0x4512d, 0x4519a, 0x4519b,
    0x115dd, 0x228da, 0x12d88, 0xd417, 0x8a32, 0x1146f, 0x115dc, 0x8aec, 0x6a0a, 0x45b2, 0x45b3, 0x4513,
    0x25b0, 0x22d8, 0x22b8, 0x2288, 0x1a83, 0x12d9, 0x116d, 0x1147, 0x116a, 0x1145, 0x968, 0x8a9,
    0x8b4, 0x6a1, 0x4b7, 0x456, 0x450, 0x22c, 0x1a9, 0x12c, 0xd5, 0x6b, 0x4a, 0x30,
    0x13, 0x7, 0x0, 0x5, 0x10, 0x19, 0x23, 0x31, 0x34, 0x44, 0x37, 0x36,
    0x24, 0x97, 0x117, 0x229, 0x351, 0x455, 0x4b5, 0x8a8, 0x969, 0x8b7, 0xd40, 0x115e,
    0x116b, 0x115f, 0x12da, 0x12db, 0x22b9, 0x22ba, 0x4518, 0x3504, 0x451a, 0x8aed, 0x4b63, 0x12d89,
    0x8a24, 0x1144a, 0x115de, 0x96c5, 0x115df, 0x45198, 0x45199, 0x4519e, 0x228db, 0x4519f, 0x4519c, 0x228d8,
    0x228d9, 0x4519d, 0x1a82d5, 0x6a0b4, 0x1a82c,
    ],
};

pub static AJOC_HCB_WET_COARSE_DF: Codebook = Codebook {
    lengths: &[
    1, 3, 4, 5, 6, 7, 8, 10, 11, 12, 12, 12, 12, 11, 10, 7, 7, 6, 5, 4, 3,
    ],
    codewords: &[
    0x0, 0x5, 0xd, 0x1e, 0x39, 0x7d, 0xf8, 0x3e4, 0x7cc, 0xf9d, 0xf9f, 0xf9c,
    0xf9e, 0x7cd, 0x3e5, 0x7f, 0x7e, 0x38, 0x1d, 0xc, 0x4,
    ],
};

pub static AJOC_HCB_WET_COARSE_DT: Codebook = Codebook {
    lengths: &[
    17, 21, 22, 22, 21, 17, 18, 16, 16, 15, 11, 13, 12, 12, 12, 11, 9, 7, 4, 3, 1, 2, 5, 7,
    9, 11, 12, 12, 12, 12, 11, 7, 16, 17, 17, 17, 21, 21, 21, 21, 21,
    ],
    codewords: &[
    0x14b7f, 0x14b7e9, 0x296fd0, 0x296fd1, 0x14b7ee, 0x14b74, 0x296fc, 0xa5bb, 0xa5be, 0x52de, 0x52b, 0x14b6,
    0xa5e, 0xa5d, 0xa55, 0x529, 0x148, 0x51, 0xb, 0x4, 0x0, 0x3, 0x15, 0x50,
    0x149, 0x528, 0xa54, 0xa5a, 0xa5c, 0xa5f, 0x52c, 0x53, 0xa5b8, 0x14b75, 0x14b72, 0x14b73,
    0x14b7ef, 0x14b7ec, 0x14b7ed, 0x14b7ea, 0x14b7eb,
    ],
};

pub static AJOC_HCB_WET_COARSE_F0: Codebook = Codebook {
    lengths: &[
    10, 14, 14, 12, 11, 11, 7, 5, 3, 3, 1, 3, 4, 6, 8, 11, 12, 13, 13, 13, 11,
    ],
    codewords: &[
    0x355, 0x356a, 0x356b, 0xd5f, 0x6ae, 0x6a8, 0x6b, 0x1b, 0x7, 0x5, 0x0, 0x4,
    0xc, 0x34, 0xd4, 0x6a9, 0xd5b, 0x1abc, 0x1ab4, 0x1abd, 0x6ac,
    ],
};

pub static AJOC_HCB_WET_FINE_DF: Codebook = Codebook {
    lengths: &[
    1, 4, 4, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 11, 11, 12, 13, 13, 14, 14, 13, 13, 14, 14,
    13, 13, 12, 11, 10, 10, 9, 8, 8, 8, 7, 7, 6, 6, 5, 4, 3,
    ],
    codewords: &[
    0x0, 0x8, 0xb, 0x15, 0x24, 0x35, 0x4d, 0x6d, 0x9d, 0xdd, 0x13c, 0x1bc,
    0x27b, 0x4f4, 0x4f5, 0x9f5, 0x13e8, 0x1bdd, 0x37b8, 0x37b9, 0x1bde, 0x1bd9, 0x37b1, 0x37b0,
    0x1bdf, 0x13e9, 0xded, 0x4fb, 0x37a, 0x27c, 0x13f, 0xdf, 0xdc, 0x9c, 0x6c, 0x4c,
    0x34, 0x25, 0x14, 0xc, 0x7,
    ],
};

pub static AJOC_HCB_WET_FINE_DT: Codebook = Codebook {
    lengths: &[
    26, 20, 26, 26, 26, 26, 26, 26, 26, 25, 19, 25, 25, 19, 19, 20, 20, 19, 18, 18, 12, 14, 14, 14,
    14, 14, 14, 14, 14, 13, 13, 12, 10, 9, 8, 7, 6, 5, 4, 3, 1, 3, 4, 5, 7, 7, 8, 9,
    11, 12, 13, 13, 14, 14, 14, 15, 14, 14, 14, 14, 12, 7, 18, 19, 19, 25, 20, 20, 20, 25, 20, 21,
    25, 25, 25, 26, 26, 26, 26, 25, 25,
    ],
    codewords: &[
    0x30968e2, 0xc25b6, 0x30968e3, 0x30968e0, 0x30968e1, 0x30968e6, 0x30968e7, 0x30968e4, 0x30968e5, 0x184b47a, 0x612de, 0x184b47b,
    0x184b478, 0x612df, 0x612dc, 0xc25b7, 0xc25b4, 0x612dd, 0x3096a, 0x3096b, 0xc58, 0x316a, 0x316c, 0x309e,
    0x316d, 0x316f, 0x3099, 0x309b, 0x3094, 0x18b4, 0x1844, 0xc24, 0x317, 0x18a, 0xc3, 0x63,
    0x32, 0x1f, 0xe, 0x4, 0x0, 0x5, 0xd, 0x1e, 0x60, 0x66, 0xc4, 0x185,
    0x610, 0xc23, 0x1845, 0x184e, 0x3095, 0x3097, 0x3098, 0x612c, 0x309f, 0x309a, 0x316e, 0x316b,
    0xc59, 0x67, 0x3096c, 0x612d0, 0x612d2, 0x184b479, 0xc25b5, 0xc25a2, 0xc25a6, 0x184b47e, 0xc25a7, 0x184b46,
    0x184b47f, 0x184b47c, 0x184b47d, 0x30968ea, 0x30968eb, 0x30968e8, 0x30968e9, 0x184b476, 0x184b477,
    ],
};

pub static AJOC_HCB_WET_FINE_F0: Codebook = Codebook {
    lengths: &[
    11, 19, 16, 17, 17, 15, 18, 14, 14, 13, 13, 12, 10, 9, 8, 7, 6, 5, 4, 3, 1, 3, 4, 5,
    6, 7, 8, 9, 10, 12, 13, 15, 15, 15, 16, 17, 15, 19, 16, 16, 12,
    ],
    codewords: &[
    0x45e, 0x454c6, 0x8a99, 0x1157c, 0x1157d, 0x455e, 0x22a62, 0x22a7, 0x22ae, 0x117d, 0x117c, 0x8a8,
    0x22b, 0x114, 0x80, 0x41, 0x21, 0x12, 0xd, 0x7, 0x0, 0x5, 0xc, 0x13,
    0x23, 0x44, 0x81, 0x116, 0x22e, 0x8aa, 0x1156, 0x4549, 0x4548, 0x454a, 0x8abf, 0x11530,
    0x454b, 0x454c7, 0x8a9a, 0x8a9b, 0x8bf,
    ],
};

pub static ASF_HCB_1: Codebook = Codebook {
    lengths: &[
    9, 9, 12, 10, 7, 10, 12, 10, 11, 10, 7, 10, 7, 5, 7, 9, 7, 10, 11, 8, 9, 9, 7, 9,
    11, 10, 12, 9, 7, 10, 7, 5, 7, 9, 7, 10, 7, 5, 7, 5, 1, 5, 7, 5, 7, 10, 7, 8,
    7, 5, 7, 10, 7, 9, 12, 10, 11, 9, 7, 10, 9, 8, 12, 10, 7, 9, 7, 5, 7, 9, 7, 10,
    12, 10, 12, 10, 7, 10, 12, 9, 9,
    ],
    codewords: &[
    0xb, 0xc, 0x0, 0x4, 0x8, 0x5, 0x1, 0x6, 0x4, 0x7, 0x9, 0x8,
    0xa, 0x8, 0xb, 0xd, 0xc, 0x9, 0x5, 0xd, 0xe, 0xf, 0xd, 0x10,
    0x6, 0xa, 0x2, 0x11, 0xe, 0xb, 0xf, 0x9, 0x10, 0x12, 0x11, 0xc,
    0x12, 0xa, 0x13, 0xb, 0x1, 0xc, 0x14, 0xd, 0x15, 0xd, 0x16, 0xe,
    0x17, 0xe, 0x18, 0xe, 0x19, 0x13, 0x3, 0xf, 0x7, 0x14, 0x1a, 0x10,
    0x15, 0xf, 0x4, 0x11, 0x1b, 0x16, 0x1c, 0xf, 0x1d, 0x17, 0x1e, 0x12,
    0x5, 0x13, 0x6, 0x14, 0x1f, 0x15, 0x7, 0x18, 0x19,
    ],
};

pub static ASF_HCB_10: Codebook = Codebook {
    lengths: &[
    6, 5, 6, 6, 7, 8, 9, 10, 10, 11, 11, 12, 12, 5, 4, 4, 5, 6, 7, 7, 8, 9, 10, 10,
    10, 11, 6, 4, 4, 5, 6, 6, 7, 8, 9, 9, 10, 10, 11, 6, 5, 5, 5, 6, 7, 7, 8, 8,
    9, 10, 10, 11, 7, 6, 6, 6, 6, 7, 7, 8, 9, 9, 10, 10, 11, 8, 7, 6, 7, 7, 7, 8,
    8, 9, 9, 10, 10, 11, 9, 7, 7, 7, 7, 8, 8, 9, 9, 10, 10, 10, 12, 10, 8, 8, 8, 8,
    8, 9, 9, 9, 10, 10, 11, 12, 10, 8, 8, 8, 8, 9, 9, 9, 10, 10, 10, 11, 12, 11, 9, 9,
    9, 9, 9, 10, 10, 10, 10, 11, 11, 13, 11, 10, 10, 10, 10, 10, 10, 10, 11, 11, 11, 11, 13, 12,
    10, 10, 10, 10, 10, 10, 10, 11, 11, 11, 12, 12, 12, 11, 11, 11, 11, 11, 11, 11, 12, 12, 13, 12,
    13,
    ],
    codewords: &[
    0x14, 0x11, 0x15, 0x16, 0x17, 0x1a, 0x1d, 0x11, 0x12, 0x7, 0x8, 0x2,
    0x3, 0x12, 0xc, 0xd, 0x13, 0x17, 0x18, 0x19, 0x1b, 0x1e, 0x13, 0x14,
    0x15, 0x9, 0x18, 0xe, 0xf, 0x14, 0x19, 0x1a, 0x1a, 0x1c, 0x1f, 0x20,
    0x16, 0x17, 0xa, 0x1b, 0x15, 0x16, 0x17, 0x1c, 0x1b, 0x1c, 0x1d, 0x1e,
    0x21, 0x18, 0x19, 0xb, 0x1d, 0x1d, 0x1e, 0x1f, 0x20, 0x1e, 0x1f, 0x1f,
    0x22, 0x23, 0x1a, 0x1b, 0xc, 0x20, 0x20, 0x21, 0x21, 0x22, 0x23, 0x21,
    0x22, 0x24, 0x25, 0x1c, 0x1d, 0xd, 0x26, 0x24, 0x25, 0x26, 0x27, 0x23,
    0x24, 0x27, 0x28, 0x1e, 0x1f, 0x20, 0x4, 0x21, 0x25, 0x26, 0x27, 0x28,
    0x29, 0x29, 0x2a, 0x2b, 0x22, 0x23, 0xe, 0x5, 0x24, 0x2a, 0x2b, 0x2c,
    0x2d, 0x2c, 0x2d, 0x2e, 0x25, 0x26, 0x27, 0xf, 0x6, 0x10, 0x2f, 0x30,
    0x31, 0x32, 0x33, 0x28, 0x29, 0x2a, 0x2b, 0x11, 0x12, 0x0, 0x13, 0x2c,
    0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x14, 0x15, 0x16, 0x17, 0x1, 0x7,
    0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x18, 0x19, 0x1a, 0x8, 0x9,
    0xa, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0xb, 0xc, 0x2, 0xd,
    0x3,
    ],
};

pub static ASF_HCB_11: Codebook = Codebook {
    lengths: &[
    3, 4, 6, 7, 8, 9, 9, 10, 10, 11, 11, 11, 12, 12, 12, 12, 10, 4, 4, 5, 6, 7, 8, 8,
    9, 9, 9, 9, 10, 10, 10, 11, 11, 9, 6, 5, 6, 6, 7, 7, 8, 8, 9, 9, 9, 9, 10, 10,
    10, 11, 8, 7, 6, 6, 7, 7, 7, 8, 8, 9, 9, 9, 9, 10, 10, 10, 11, 8, 8, 7, 7, 7,
    7, 8, 8, 9, 9, 9, 9, 10, 10, 10, 10, 11, 8, 9, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10,
    10, 10, 10, 11, 11, 8, 9, 8, 8, 8, 8, 8, 9, 9, 9, 10, 10, 10, 10, 10, 11, 11, 8, 10,
    9, 8, 8, 8, 9, 9, 9, 9, 10, 10, 10, 11, 11, 11, 11, 9, 10, 9, 9, 9, 9, 9, 9, 9,
    10, 10, 10, 10, 11, 11, 11, 11, 9, 11, 9, 9, 9, 9, 9, 10, 10, 10, 10, 10, 10, 11, 11, 11,
    12, 9, 11, 9, 9, 9, 9, 10, 10, 10, 10, 10, 10, 11, 11, 11, 11, 12, 9, 11, 10, 9, 9, 9,
    10, 10, 10, 10, 10, 11, 11, 11, 11, 11, 12, 9, 12, 10, 10, 10, 10, 10, 10, 10, 11, 11, 11, 11,
    11, 11, 12, 12, 9, 11, 10, 10, 10, 10, 10, 10, 10, 11, 11, 11, 11, 11, 12, 12, 12, 9, 12, 11,
    10, 10, 10, 11, 11, 11, 11, 11, 11, 11, 12, 12, 12, 12, 9, 12, 11, 11, 11, 11, 11, 11, 11, 11,
    12, 12, 12, 12, 12, 12, 12, 9, 10, 8, 8, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9, 9, 9, 9,
    5,
    ],
    codewords: &[
    0x7, 0xb, 0x1f, 0x2f, 0x3d, 0x36, 0x37, 0x26, 0x27, 0xd, 0xe, 0xf,
    0x0, 0x1, 0x2, 0x3, 0x28, 0xc, 0xd, 0x13, 0x20, 0x30, 0x3e, 0x3f,
    0x38, 0x39, 0x3a, 0x3b, 0x29, 0x2a, 0x2b, 0x10, 0x11, 0x3c, 0x21, 0x14,
    0x22, 0x23, 0x31, 0x32, 0x40, 0x41, 0x3d, 0x3e, 0x3f, 0x40, 0x2c, 0x2d,
    0x2e, 0x12, 0x42, 0x33, 0x24, 0x25, 0x34, 0x35, 0x36, 0x43, 0x44, 0x41,
    0x42, 0x43, 0x44, 0x2f, 0x30, 0x31, 0x13, 0x45, 0x46, 0x37, 0x38, 0x39,
    0x3a, 0x47, 0x48, 0x45, 0x46, 0x47, 0x48, 0x32, 0x33, 0x34, 0x35, 0x14,
    0x49, 0x49, 0x3b, 0x3c, 0x3d, 0x4a, 0x4b, 0x4c, 0x4a, 0x4b, 0x4c, 0x36,
    0x37, 0x38, 0x39, 0x15, 0x16, 0x4d, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52,
    0x4e, 0x4f, 0x50, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x17, 0x18, 0x53, 0x3f,
    0x51, 0x54, 0x55, 0x56, 0x52, 0x53, 0x54, 0x55, 0x40, 0x41, 0x42, 0x19,
    0x1a, 0x1b, 0x1c, 0x56, 0x43, 0x57, 0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5d,
    0x44, 0x45, 0x46, 0x47, 0x1d, 0x1e, 0x1f, 0x20, 0x5e, 0x21, 0x5f, 0x60,
    0x61, 0x62, 0x63, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x22, 0x23, 0x24,
    0x4, 0x64, 0x25, 0x65, 0x66, 0x67, 0x68, 0x4e, 0x4f, 0x50, 0x51, 0x52,
    0x53, 0x26, 0x27, 0x28, 0x29, 0x5, 0x69, 0x2a, 0x54, 0x6a, 0x6b, 0x6c,
    0x55, 0x56, 0x57, 0x58, 0x59, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x6, 0x6d,
    0x7, 0x5a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f, 0x60, 0x30, 0x31, 0x32, 0x33,
    0x34, 0x35, 0x8, 0x9, 0x6e, 0x36, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66,
    0x67, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0xa, 0xb, 0xc, 0x6f, 0xd, 0x3c,
    0x68, 0x69, 0x6a, 0x3d, 0x3e, 0x3f, 0x40, 0x41, 0x42, 0x43, 0xe, 0xf,
    0x10, 0x11, 0x70, 0x12, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b,
    0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x71, 0x6b, 0x57, 0x58, 0x59,
    0x5a, 0x5b, 0x5c, 0x5d, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79,
    0x15,
    ],
};

pub static ASF_HCB_2: Codebook = Codebook {
    lengths: &[
    8, 7, 9, 8, 6, 8, 9, 8, 9, 8, 6, 7, 6, 5, 6, 7, 6, 8, 9, 7, 7, 7, 6, 7,
    9, 8, 9, 8, 6, 8, 6, 5, 6, 7, 6, 8, 6, 5, 6, 5, 3, 5, 6, 5, 6, 8, 6, 7,
    6, 5, 6, 8, 6, 8, 9, 8, 9, 8, 6, 7, 7, 7, 9, 8, 6, 7, 6, 4, 6, 8, 6, 8,
    9, 8, 9, 8, 6, 8, 9, 7, 8,
    ],
    codewords: &[
    0x6, 0xe, 0x0, 0x7, 0xe, 0x8, 0x1, 0x9, 0x2, 0xa, 0xf, 0xf,
    0x10, 0x13, 0x11, 0x10, 0x12, 0xb, 0x3, 0x11, 0x12, 0x13, 0x13, 0x14,
    0x4, 0xc, 0x5, 0xd, 0x14, 0xe, 0x15, 0x14, 0x16, 0x15, 0x17, 0xf,
    0x18, 0x15, 0x19, 0x16, 0x7, 0x17, 0x1a, 0x18, 0x1b, 0x10, 0x1c, 0x16,
    0x1d, 0x19, 0x1e, 0x11, 0x1f, 0x12, 0x6, 0x13, 0x7, 0x14, 0x20, 0x17,
    0x18, 0x19, 0x8, 0x15, 0x21, 0x1a, 0x22, 0xd, 0x23, 0x16, 0x24, 0x17,
    0x9, 0x18, 0xa, 0x19, 0x25, 0x1a, 0xb, 0x1b, 0x1b,
    ],
};

pub static ASF_HCB_3: Codebook = Codebook {
    lengths: &[
    1, 4, 9, 4, 5, 9, 10, 10, 12, 4, 6, 10, 5, 6, 9, 10, 9, 11, 10, 11, 14, 10, 10, 12,
    12, 11, 13, 4, 6, 11, 6, 7, 10, 11, 10, 12, 5, 7, 11, 6, 6, 10, 10, 9, 11, 9, 10, 13,
    9, 9, 12, 11, 11, 12, 9, 10, 15, 10, 11, 15, 14, 13, 15, 9, 10, 14, 9, 10, 13, 13, 12, 14,
    11, 12, 15, 11, 11, 14, 13, 12, 14,
    ],
    codewords: &[
    0x1, 0x4, 0xd, 0x5, 0x5, 0xe, 0xa, 0xb, 0x5, 0x6, 0x4, 0xc,
    0x6, 0x5, 0xf, 0xd, 0x10, 0x7, 0xe, 0x8, 0x2, 0xf, 0x10, 0x6,
    0x7, 0x9, 0x4, 0x7, 0x6, 0xa, 0x7, 0x6, 0x11, 0xb, 0x12, 0x8,
    0x7, 0x7, 0xc, 0x8, 0x9, 0x13, 0x14, 0x11, 0xd, 0x12, 0x15, 0x5,
    0x13, 0x14, 0x9, 0xe, 0xf, 0xa, 0x15, 0x16, 0x0, 0x17, 0x10, 0x1,
    0x3, 0x6, 0x2, 0x16, 0x18, 0x4, 0x17, 0x19, 0x7, 0x8, 0xb, 0x5,
    0x11, 0xc, 0x3, 0x12, 0x13, 0x6, 0x9, 0xd, 0x7,
    ],
};

pub static ASF_HCB_4: Codebook = Codebook {
    lengths: &[
    4, 4, 9, 5, 4, 8, 9, 8, 11, 4, 5, 8, 5, 4, 8, 8, 8, 10, 9, 9, 11, 8, 8, 10,
    11, 10, 12, 4, 5, 8, 5, 4, 8, 9, 8, 10, 4, 4, 8, 4, 4, 7, 8, 7, 9, 8, 8, 11,
    8, 7, 10, 10, 9, 10, 9, 8, 11, 8, 8, 11, 11, 10, 12, 8, 8, 11, 8, 7, 10, 10, 9, 11,
    11, 10, 12, 10, 9, 11, 12, 10, 11,
    ],
    codewords: &[
    0x5, 0x6, 0xa, 0x5, 0x7, 0xa, 0xb, 0xb, 0x2, 0x8, 0x6, 0xc,
    0x7, 0x9, 0xd, 0xe, 0xf, 0x7, 0xc, 0xd, 0x3, 0x10, 0x11, 0x8,
    0x4, 0x9, 0x0, 0xa, 0x8, 0x12, 0x9, 0xb, 0x13, 0xe, 0x14, 0xa,
    0xc, 0xd, 0x15, 0xe, 0xf, 0x10, 0x16, 0x11, 0xf, 0x17, 0x18, 0x5,
    0x19, 0x12, 0xb, 0xc, 0x10, 0xd, 0x11, 0x1a, 0x6, 0x1b, 0x1c, 0x7,
    0x8, 0xe, 0x1, 0x1d, 0x1e, 0x9, 0x1f, 0x13, 0xf, 0x10, 0x12, 0xa,
    0xb, 0x11, 0x2, 0x12, 0x13, 0xc, 0x3, 0x13, 0xd,
    ],
};

pub static ASF_HCB_5: Codebook = Codebook {
    lengths: &[
    14, 13, 12, 12, 11, 11, 12, 13, 14, 13, 12, 10, 9, 9, 9, 10, 11, 13, 13, 10, 9, 8, 7, 7,
    9, 10, 12, 12, 9, 8, 5, 4, 5, 7, 9, 12, 12, 9, 7, 4, 1, 4, 7, 9, 12, 11, 9, 7,
    5, 4, 5, 8, 9, 12, 12, 10, 9, 7, 7, 8, 9, 10, 12, 13, 11, 10, 9, 9, 10, 10, 12, 13,
    14, 12, 12, 12, 11, 11, 12, 13, 14,
    ],
    codewords: &[
    0x0, 0x2, 0x5, 0x6, 0xb, 0xc, 0x7, 0x3, 0x1, 0x4, 0x8, 0x9,
    0x9, 0xa, 0xb, 0xa, 0xd, 0x5, 0x6, 0xb, 0xc, 0xc, 0x8, 0x9,
    0xd, 0xc, 0x9, 0xa, 0xe, 0xd, 0x4, 0x4, 0x5, 0xa, 0xf, 0xb,
    0xc, 0x10, 0xb, 0x5, 0x1, 0x6, 0xc, 0x11, 0xd, 0xe, 0x12, 0xd,
    0x6, 0x7, 0x7, 0xe, 0x13, 0xe, 0xf, 0xd, 0x14, 0xe, 0xf, 0xf,
    0x15, 0xe, 0x10, 0x7, 0xf, 0xf, 0x16, 0x17, 0x10, 0x11, 0x11, 0x8,
    0x2, 0x12, 0x13, 0x14, 0x10, 0x11, 0x15, 0x9, 0x3,
    ],
};

pub static ASF_HCB_6: Codebook = Codebook {
    lengths: &[
    12, 11, 10, 10, 10, 10, 10, 11, 12, 11, 9, 8, 8, 8, 8, 8, 9, 11, 10, 8, 6, 6, 6, 5,
    6, 8, 10, 10, 8, 6, 4, 4, 4, 5, 8, 10, 10, 8, 6, 4, 4, 4, 6, 8, 10, 10, 8, 5,
    4, 4, 4, 6, 8, 10, 10, 8, 6, 5, 6, 5, 6, 8, 11, 11, 9, 8, 8, 8, 8, 8, 9, 11,
    12, 11, 10, 10, 11, 10, 10, 11, 12,
    ],
    codewords: &[
    0x0, 0x2, 0x6, 0x7, 0x8, 0x9, 0xa, 0x3, 0x1, 0x4, 0xc, 0x8,
    0x9, 0xa, 0xb, 0xc, 0xd, 0x5, 0xb, 0xd, 0x7, 0x8, 0x9, 0x9,
    0xa, 0xe, 0xc, 0xd, 0xf, 0xb, 0x7, 0x8, 0x9, 0xa, 0x10, 0xe,
    0xf, 0x11, 0xc, 0xa, 0xb, 0xc, 0xd, 0x12, 0x10, 0x11, 0x13, 0xb,
    0xd, 0xe, 0xf, 0xe, 0x14, 0x12, 0x13, 0x15, 0xf, 0xc, 0x10, 0xd,
    0x11, 0x16, 0x6, 0x7, 0xe, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0xf, 0x8,
    0x2, 0x9, 0x14, 0x15, 0xa, 0x16, 0x17, 0xb, 0x3,
    ],
};

pub static ASF_HCB_7: Codebook = Codebook {
    lengths: &[
    1, 3, 6, 8, 9, 9, 10, 11, 3, 4, 6, 7, 8, 8, 9, 10, 6, 5, 7, 8, 8, 9, 10, 11,
    8, 7, 8, 8, 9, 9, 10, 11, 9, 8, 8, 9, 10, 10, 10, 11, 9, 8, 9, 9, 10, 10, 11, 12,
    10, 9, 10, 10, 10, 11, 12, 12, 12, 10, 11, 11, 12, 12, 12, 12,
    ],
    codewords: &[
    0x1, 0x2, 0x7, 0xb, 0xa, 0xb, 0x6, 0x4, 0x3, 0x3, 0x8, 0xb,
    0xc, 0xd, 0xc, 0x7, 0x9, 0x5, 0xc, 0xe, 0xf, 0xd, 0x8, 0x5,
    0x10, 0xd, 0x11, 0x12, 0xe, 0xf, 0x9, 0x6, 0x10, 0x13, 0x14, 0x11,
    0xa, 0xb, 0xc, 0x7, 0x12, 0x15, 0x13, 0x14, 0xd, 0xe, 0x8, 0x0,
    0xf, 0x15, 0x10, 0x11, 0x12, 0x9, 0x1, 0x2, 0x3, 0x13, 0xa, 0xb,
    0x4, 0x5, 0x6, 0x7,
    ],
};

pub static ASF_HCB_8: Codebook = Codebook {
    lengths: &[
    4, 4, 5, 6, 7, 8, 9, 10, 4, 3, 4, 5, 6, 7, 8, 9, 5, 4, 4, 5, 6, 7, 8, 9,
    6, 5, 5, 5, 6, 7, 8, 9, 7, 6, 6, 6, 7, 7, 9, 10, 8, 7, 7, 7, 7, 8, 9, 10,
    9, 8, 8, 8, 8, 9, 9, 10, 11, 9, 9, 9, 10, 10, 10, 11,
    ],
    codewords: &[
    0x8, 0x9, 0x9, 0xa, 0x9, 0x8, 0x4, 0x1, 0xa, 0x7, 0xb, 0xa,
    0xb, 0xa, 0x9, 0x5, 0xb, 0xc, 0xd, 0xc, 0xc, 0xb, 0xa, 0x6,
    0xd, 0xd, 0xe, 0xf, 0xe, 0xc, 0xb, 0x7, 0xd, 0xf, 0x10, 0x11,
    0xe, 0xf, 0x8, 0x2, 0xc, 0x10, 0x11, 0x12, 0x13, 0xd, 0x9, 0x3,
    0xa, 0xe, 0xf, 0x10, 0x11, 0xb, 0xc, 0x4, 0x0, 0xd, 0xe, 0xf,
    0x5, 0x6, 0x7, 0x1,
    ],
};

pub static ASF_HCB_9: Codebook = Codebook {
    lengths: &[
    1, 3, 6, 8, 9, 10, 10, 11, 11, 12, 12, 12, 13, 3, 4, 6, 7, 8, 9, 9, 10, 10, 11, 11,
    11, 12, 6, 6, 7, 8, 9, 9, 10, 10, 10, 11, 11, 12, 12, 8, 7, 8, 9, 9, 10, 10, 11, 11,
    11, 12, 12, 12, 9, 8, 9, 9, 10, 10, 11, 11, 11, 12, 12, 12, 13, 10, 9, 9, 10, 10, 10, 11,
    11, 11, 12, 12, 12, 13, 10, 9, 10, 10, 11, 11, 11, 12, 12, 12, 12, 13, 13, 11, 10, 10, 11, 11,
    12, 12, 12, 12, 12, 13, 13, 13, 11, 10, 10, 11, 11, 11, 12, 12, 12, 13, 13, 13, 14, 12, 11, 11,
    11, 11, 12, 12, 12, 13, 13, 13, 14, 14, 12, 11, 11, 12, 12, 12, 12, 13, 13, 13, 13, 14, 14, 12,
    11, 11, 12, 12, 12, 13, 13, 13, 13, 13, 14, 14, 13, 12, 12, 12, 13, 13, 13, 13, 14, 14, 14, 14,
    14,
    ],
    codewords: &[
    0x1, 0x2, 0x8, 0x14, 0x1b, 0x20, 0x21, 0x1e, 0x1f, 0x11, 0x12, 0x13,
    0x6, 0x3, 0x3, 0x9, 0xd, 0x15, 0x1c, 0x1d, 0x22, 0x23, 0x20, 0x21,
    0x22, 0x14, 0xa, 0xb, 0xe, 0x16, 0x1e, 0x1f, 0x24, 0x25, 0x26, 0x23,
    0x24, 0x15, 0x16, 0x17, 0xf, 0x18, 0x20, 0x21, 0x27, 0x28, 0x25, 0x26,
    0x27, 0x17, 0x18, 0x19, 0x22, 0x19, 0x23, 0x24, 0x29, 0x2a, 0x28, 0x29,
    0x2a, 0x1a, 0x1b, 0x1c, 0x7, 0x2b, 0x25, 0x26, 0x2c, 0x2d, 0x2e, 0x2b,
    0x2c, 0x2d, 0x1d, 0x1e, 0x1f, 0x8, 0x2f, 0x27, 0x30, 0x31, 0x2e, 0x2f,
    0x30, 0x20, 0x21, 0x22, 0x23, 0x9, 0xa, 0x31, 0x32, 0x33, 0x32, 0x33,
    0x24, 0x25, 0x26, 0x27, 0x28, 0xb, 0xc, 0xd, 0x34, 0x34, 0x35, 0x35,
    0x36, 0x37, 0x29, 0x2a, 0x2b, 0xe, 0xf, 0x10, 0x0, 0x2c, 0x38, 0x39,
    0x3a, 0x3b, 0x2d, 0x2e, 0x2f, 0x11, 0x12, 0x13, 0x1, 0x2, 0x30, 0x3c,
    0x3d, 0x31, 0x32, 0x33, 0x34, 0x14, 0x15, 0x16, 0x17, 0x3, 0x4, 0x35,
    0x3e, 0x3f, 0x36, 0x37, 0x38, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x5, 0x6,
    0x1d, 0x39, 0x3a, 0x3b, 0x1e, 0x1f, 0x20, 0x21, 0x7, 0x8, 0x9, 0xa,
    0xb,
    ],
};

pub static ASF_HCB_SCALEFAC: Codebook = Codebook {
    lengths: &[
    17, 14, 14, 14, 14, 14, 14, 15, 14, 14, 14, 14, 14, 14, 14, 14, 13, 13, 14, 14, 14, 14, 14, 13,
    14, 13, 13, 14, 14, 13, 14, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 12, 12, 12, 12, 11,
    11, 11, 10, 9, 9, 8, 8, 6, 6, 4, 4, 3, 1, 4, 4, 5, 6, 7, 8, 8, 9, 9, 10, 10,
    11, 11, 11, 11, 12, 11, 12, 12, 12, 12, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 14, 14, 14, 14,
    14, 13, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 16, 14, 14,
    17,
    ],
    codewords: &[
    0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x1, 0x7, 0x8, 0x9, 0xa,
    0xb, 0xc, 0xd, 0xe, 0x19, 0x1a, 0xf, 0x10, 0x11, 0x12, 0x13, 0x1b,
    0x14, 0x1c, 0x1d, 0x15, 0x16, 0x1e, 0x17, 0x1f, 0x20, 0x21, 0x22, 0x23,
    0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x1b, 0x1c, 0x1d, 0x1e, 0x12,
    0x13, 0x14, 0xd, 0x8, 0x9, 0x6, 0x7, 0x3, 0x4, 0x2, 0x3, 0x3,
    0x1, 0x4, 0x5, 0x3, 0x5, 0x5, 0x8, 0x9, 0xa, 0xb, 0xe, 0xf,
    0x15, 0x16, 0x17, 0x18, 0x1f, 0x19, 0x20, 0x21, 0x22, 0x23, 0x2b, 0x2c,
    0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x18, 0x19, 0x1a, 0x1b,
    0x1c, 0x35, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26,
    0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x1, 0x30, 0x31,
    0x1,
    ],
};

pub static ASF_HCB_SNF: Codebook = Codebook {
    lengths: &[
    4, 7, 8, 8, 7, 6, 6, 6, 6, 5, 5, 4, 4, 3, 3, 3, 3, 4, 4, 6, 7, 6,
    ],
    codewords: &[
    0x3, 0x1, 0x0, 0x1, 0x2, 0x2, 0x3, 0x4, 0x5, 0x4, 0x5, 0x4,
    0x5, 0x4, 0x5, 0x6, 0x7, 0x6, 0x7, 0x6, 0x3, 0x7,
    ],
};

pub static ASPX_HCB_ENV_BALANCE_15_DF: Codebook = Codebook {
    lengths: &[
    19, 19, 19, 19, 19, 18, 18, 17, 16, 16, 16, 15, 13, 13, 12, 12, 12, 11, 10, 9, 8, 7, 5, 3,
    1, 2, 4, 7, 8, 10, 10, 11, 12, 12, 13, 13, 14, 15, 16, 18, 18, 17, 18, 18, 18, 19, 19, 18,
    19,
    ],
    codewords: &[
    0x7def6, 0x7def7, 0x7defa, 0x7defb, 0x7dffe, 0x3ef7c, 0x3ef7e, 0x1f7b2, 0xfbd8, 0xfbda, 0xfbfe, 0x7dfe,
    0x1f6e, 0x1f7a, 0xf9e, 0xfb6, 0xfbc, 0x7da, 0x3ec, 0x1f2, 0xfa, 0x7e, 0x1e, 0x6,
    0x0, 0x2, 0xe, 0x7f, 0xf8, 0x3ee, 0x3e6, 0x7ce, 0xfbe, 0xf9f, 0x1f7e, 0x1f6f,
    0x3efe, 0x7dee, 0xfbdb, 0x3ef78, 0x3ef7f, 0x1f7b3, 0x3effc, 0x3effd, 0x3ef79, 0x7dffc, 0x7dffd, 0x3ef7a,
    0x7dfff,
    ],
};

pub static ASPX_HCB_ENV_BALANCE_15_DT: Codebook = Codebook {
    lengths: &[
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 14, 13, 13, 12, 12, 11, 12, 11, 9, 8, 7, 4, 2,
    1, 3, 5, 7, 9, 9, 10, 11, 11, 12, 12, 12, 12, 13, 15, 14, 15, 14, 15, 15, 15, 14, 14, 15,
    15,
    ],
    codewords: &[
    0x7dce, 0x7dcf, 0x7ddc, 0x7ddd, 0x7dde, 0x7ddf, 0x7dec, 0x7ded, 0x7dee, 0x7def, 0x7df8, 0x3ebe,
    0x1f5e, 0x1f7a, 0xfb8, 0xfba, 0x7cc, 0xfbe, 0x7d6, 0x1f2, 0xf8, 0x7e, 0xe, 0x2,
    0x0, 0x6, 0x1e, 0x7f, 0x1f6, 0x1f4, 0x3ea, 0x7cd, 0x7ce, 0xf9e, 0xfae, 0xfbc,
    0xf9f, 0x1f76, 0x7df9, 0x3ebf, 0x7dfa, 0x3ee4, 0x7dfb, 0x7dfc, 0x7dfd, 0x3ee5, 0x3ee6, 0x7dfe,
    0x7dff,
    ],
};

pub static ASPX_HCB_ENV_BALANCE_15_F0: Codebook = Codebook {
    lengths: &[
    13, 16, 15, 15, 13, 12, 11, 9, 8, 7, 5, 3, 1, 2, 4, 7, 8, 9, 10, 10, 11, 12, 13, 15,
    16,
    ],
    codewords: &[
    0x1ffe, 0xffea, 0x7ff6, 0x7ff7, 0x1fff, 0xffc, 0x7de, 0x1f6, 0xfa, 0x7c, 0x1e, 0x6,
    0x0, 0x2, 0xe, 0x7e, 0xfe, 0x1fe, 0x3fe, 0x3ee, 0x7df, 0xffd, 0x1ffc, 0x7ff4,
    0xffeb,
    ],
};

pub static ASPX_HCB_ENV_BALANCE_30_DF: Codebook = Codebook {
    lengths: &[
    17, 17, 17, 15, 14, 12, 9, 9, 8, 7, 5, 2, 1, 3, 4, 7, 8, 9, 10, 11, 14, 15, 16, 16,
    17,
    ],
    codewords: &[
    0x1fffc, 0x1fffd, 0x1fffe, 0x7ffa, 0x3ffc, 0xffe, 0x1fc, 0x1fe, 0xfa, 0x7e, 0x1e, 0x2,
    0x0, 0x6, 0xe, 0x7c, 0xfb, 0x1fd, 0x3fe, 0x7fe, 0x3ffe, 0x7ffb, 0xfffc, 0xfffd,
    0x1ffff,
    ],
};

pub static ASPX_HCB_ENV_BALANCE_30_DT: Codebook = Codebook {
    lengths: &[
    15, 15, 15, 14, 13, 13, 10, 8, 8, 7, 4, 2, 1, 3, 5, 7, 9, 9, 9, 12, 14, 13, 14, 15,
    14,
    ],
    codewords: &[
    0x7ff2, 0x7ff3, 0x7ff6, 0x3ff6, 0x1ffe, 0x1fff, 0x3fe, 0xfa, 0xfe, 0x7e, 0xe, 0x2,
    0x0, 0x6, 0x1e, 0x7c, 0x1fe, 0x1f6, 0x1f7, 0xffc, 0x3ffa, 0x1ffa, 0x3ff7, 0x7ff7,
    0x3ff8,
    ],
};

pub static ASPX_HCB_ENV_BALANCE_30_F0: Codebook = Codebook {
    lengths: &[
    12, 11, 9, 7, 4, 3, 1, 2, 5, 6, 8, 10, 12,
    ],
    codewords: &[
    0xffe, 0x7fe, 0x1fe, 0x7e, 0xe, 0x6, 0x0, 0x2, 0x1e, 0x3e, 0xfe, 0x3fe,
    0xfff,
    ],
};

pub static ASPX_HCB_ENV_LEVEL_15_DF: Codebook = Codebook {
    lengths: &[
    21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 17, 18, 19, 19, 21, 19,
    19, 19, 21, 20, 20, 20, 16, 17, 19, 19, 18, 18, 17, 15, 15, 15, 15, 14, 14, 14, 14, 13, 13, 13,
    13, 13, 13, 13, 12, 12, 12, 12, 12, 11, 10, 10, 9, 8, 8, 7, 6, 6, 5, 4, 3, 2, 2, 3,
    4, 6, 7, 7, 8, 9, 10, 11, 11, 12, 13, 13, 14, 14, 15, 15, 15, 16, 16, 16, 17, 17, 17, 17,
    17, 18, 17, 17, 16, 17, 18, 19, 20, 21, 19, 19, 18, 16, 19, 21, 16, 20, 20, 21, 21, 20, 20, 19,
    20, 19, 17, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21,
    ],
    codewords: &[
    0xfeff2, 0xfeff3, 0xfeff4, 0xfeff5, 0xfeff6, 0xfeff7, 0xfeff8, 0xfeff9, 0xfeffa, 0xfeffb, 0xfeffc, 0xfeffd,
    0xfeffe, 0xfefff, 0xff5e4, 0xff5e5, 0xff5e6, 0xff5e7, 0xfef4, 0x1fefe, 0x3fdea, 0x3ffde, 0xff7b6, 0x3fdec,
    0x3fdee, 0x3fd78, 0xff5ee, 0x7f798, 0x7faf6, 0x7f799, 0x7ffa, 0xfef6, 0x3fdef, 0x3ffdf, 0x1fdee, 0x1ffee,
    0xff7e, 0x3d7e, 0x3fba, 0x3fbe, 0x3ffe, 0x1ebe, 0x1fdc, 0x1fee, 0x1ff6, 0xf5c, 0xf5d, 0xf7e,
    0xf7f, 0xff4, 0xff6, 0xffe, 0x7bc, 0x7bd, 0x7f6, 0x7fc, 0x7fe, 0x3fa, 0x1ea, 0x1fc,
    0xf6, 0x76, 0x7e, 0x3c, 0x1c, 0x3e, 0x1e, 0xe, 0x6, 0x2, 0x0, 0x2,
    0x6, 0x3f, 0x3e, 0x3a, 0x77, 0xf4, 0x1ee, 0x3fc, 0x3d6, 0x7be, 0xffa, 0xf5e,
    0x1ff7, 0x1fea, 0x3ffc, 0x3fd6, 0x3fbb, 0x7fbe, 0x7fae, 0x7f7e, 0xfff6, 0xfffe, 0xfef5, 0xfef2,
    0xf5fe, 0x1fffe, 0xff78, 0xfefe, 0x7afe, 0xff79, 0x1fef4, 0x3fd7a, 0x7f79a, 0xff5ef, 0x3fdfe, 0x3fffe,
    0x1fdef, 0x7ffe, 0x3fdff, 0xff5f2, 0x7f78, 0x7f79b, 0x7f79e, 0xff5f3, 0xff5f4, 0x7f79f, 0x7faf8, 0x3ffff,
    0x7f7f8, 0x3fbce, 0xf5ff, 0xff5f5, 0xff5f6, 0xff5f7, 0xff5f8, 0xff5f9, 0xff5fa, 0xff5fb, 0xff5fc, 0xff5fd,
    0xff5fe, 0xff5ff, 0xff7ac, 0xff7ad, 0xff7ae, 0xff7af, 0xff7b4, 0xff7b5, 0xff7b7,
    ],
};

pub static ASPX_HCB_ENV_LEVEL_15_DT: Codebook = Codebook {
    lengths: &[
    19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19,
    19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 18, 18, 18, 18, 18,
    17, 19, 17, 16, 17, 17, 17, 16, 15, 16, 15, 15, 14, 14, 14, 13, 12, 10, 8, 6, 4, 2, 1, 3,
    5, 7, 10, 11, 12, 14, 14, 15, 15, 16, 16, 17, 16, 17, 17, 18, 16, 17, 18, 18, 19, 18, 19, 19,
    17, 18, 18, 19, 19, 19, 19, 19, 19, 19, 19, 19, 18, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19,
    19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19,
    ],
    codewords: &[
    0x7ffa0, 0x7ffa1, 0x7ffa2, 0x7ffa3, 0x7ffa4, 0x7ffa5, 0x7ffa6, 0x7ffa7, 0x7ffa8, 0x7ffa9, 0x7ffaa, 0x7ffab,
    0x7ffac, 0x7ffad, 0x7ffae, 0x7ffaf, 0x7ffb0, 0x7ffb1, 0x7ffb2, 0x7ffb3, 0x7ffb4, 0x7ffb5, 0x7ffb6, 0x7ffb7,
    0x7ffb8, 0x7ffb9, 0x7ffba, 0x7ffbb, 0x7ffbc, 0x7ffbd, 0x7ffbe, 0x7ffbf, 0x7ffc0, 0x7ffc1, 0x7ffc2, 0x7ffc3,
    0x7ffc4, 0x7ffc5, 0x7ffc6, 0x7ffc7, 0x7ffcc, 0x7ffcd, 0x7ffce, 0x3ff16, 0x3ff17, 0x3ff1e, 0x3ff1f, 0x3ff3c,
    0x1fefa, 0x7ffcf, 0x1ffb6, 0xff7c, 0x1ff8e, 0x1ffbe, 0x1fefb, 0xffc4, 0x7fba, 0xffc6, 0x7fbb, 0x7fee,
    0x3fdc, 0x3ff0, 0x3ff2, 0x1ffa, 0xffe, 0x3fc, 0xfe, 0x3e, 0xe, 0x2, 0x0, 0x6,
    0x1e, 0x7e, 0x3fe, 0x7fa, 0xff6, 0x3ffc, 0x3fde, 0x7fe6, 0x7fec, 0xffda, 0xffde, 0x1fefe,
    0xffce, 0x1fff2, 0x1ffbf, 0x3ff3d, 0xff7e, 0x1feff, 0x3ff3e, 0x3ff3f, 0x7ffd2, 0x3ffe8, 0x7ffd3, 0x7ffd4,
    0x1ff8a, 0x3ff6e, 0x3ff6f, 0x7ffd5, 0x7ffd6, 0x7ffd7, 0x7ffd8, 0x7ffd9, 0x7ffda, 0x7ffdb, 0x7ffde, 0x7ffdf,
    0x3ffee, 0x7ffe0, 0x7ffe1, 0x7ffe2, 0x7ffe3, 0x7ffe4, 0x7ffe5, 0x7ffe6, 0x7ffe7, 0x7ffe8, 0x7ffe9, 0x7ffea,
    0x7ffeb, 0x7ffec, 0x7ffed, 0x7ffee, 0x7ffef, 0x7fff0, 0x7fff1, 0x7fff2, 0x7fff3, 0x7fff4, 0x7fff5, 0x7fff6,
    0x7fff7, 0x7fff8, 0x7fff9, 0x7fffa, 0x7fffb, 0x7fffc, 0x7fffd, 0x7fffe, 0x7ffff,
    ],
};

pub static ASPX_HCB_ENV_LEVEL_15_F0: Codebook = Codebook {
    lengths: &[
    7, 9, 9, 9, 9, 8, 8, 7, 7, 7, 7, 7, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 5, 5,
    5, 5, 5, 5, 5, 5, 4, 5, 5, 5, 5, 5, 5, 5, 4, 5, 5, 5, 5, 6, 6, 6, 7, 8,
    9, 10, 11, 12, 14, 14, 16, 16, 17, 17, 18, 18, 18, 18, 18, 18, 18, 18, 17, 17, 17, 17, 16,
    ],
    codewords: &[
    0x3e, 0x7e, 0x7f, 0xbe, 0xbf, 0x3e, 0x5e, 0x16, 0x17, 0x1e, 0x2e, 0x5e,
    0x12, 0xa, 0xc, 0xe, 0x13, 0x16, 0x1a, 0x1b, 0x22, 0x23, 0x4, 0x8,
    0xc, 0xe, 0x12, 0x14, 0x15, 0x1a, 0x0, 0x16, 0x18, 0x1e, 0x1c, 0x19,
    0x1d, 0x1f, 0x1, 0x1b, 0x13, 0x10, 0xa, 0x2e, 0x1e, 0xd, 0x3f, 0xbe,
    0x17e, 0x2fe, 0x5fe, 0xbfe, 0x2ffe, 0x2ffc, 0xbffc, 0xbff6, 0x17ffa, 0x17ffb, 0x2fff8, 0x2fff9,
    0x2fffa, 0x2fffb, 0x2fffc, 0x2fffd, 0x2fffe, 0x2ffff, 0x17fe8, 0x17fe9, 0x17fea, 0x17feb, 0xbff7,
    ],
};

pub static ASPX_HCB_ENV_LEVEL_30_DF: Codebook = Codebook {
    lengths: &[
    23, 23, 23, 23, 22, 22, 22, 22, 20, 19, 19, 20, 19, 18, 17, 16, 16, 15, 14, 14, 14, 13, 13, 13,
    13, 12, 12, 12, 11, 10, 9, 8, 5, 4, 2, 1, 3, 6, 8, 9, 11, 12, 14, 14, 15, 16, 17, 17,
    17, 18, 18, 19, 21, 19, 17, 20, 19, 20, 20, 20, 19, 19, 19, 22, 22, 22, 22, 22, 22, 22, 22,
    ],
    codewords: &[
    0x7ffbfc, 0x7ffbfd, 0x7ffbfe, 0x7ffbff, 0x3f7f6c, 0x3f7f6d, 0x3f7f6e, 0x3f7f6f, 0xfdfee, 0x7effe, 0x7efec, 0xfff7e,
    0x7ffbe, 0x3f7fe, 0x1fbfe, 0xfdfc, 0xfff6, 0x7ffa, 0x3f7e, 0x3ffc, 0x3ffe, 0x1fba, 0x1fbe, 0x1ffa,
    0x1ffb, 0xfdc, 0xffc, 0xffe, 0x7fc, 0x3f6, 0x1fe, 0xfe, 0x1e, 0xe, 0x2, 0x0,
    0x6, 0x3e, 0xfc, 0x1fa, 0x7fd, 0xfde, 0x3fff, 0x3f76, 0x7eee, 0xfdde, 0x1ffee, 0x1fbfc,
    0x1fbfa, 0x3ffde, 0x3f77e, 0x7efee, 0x1ffefe, 0x7eefe, 0x1fbbe, 0xfdfea, 0x7eff6, 0xfdfda, 0xfdfde, 0xfdfef,
    0x7eff4, 0x7efff, 0x7eeff, 0x3f7fae, 0x3f7f7c, 0x3f7f7d, 0x3f7f7e, 0x3f7f7f, 0x3f7fac, 0x3f7fad, 0x3f7faf,
    ],
};

pub static ASPX_HCB_ENV_LEVEL_30_DT: Codebook = Codebook {
    lengths: &[
    20, 20, 20, 20, 20, 20, 20, 20, 20, 19, 19, 20, 20, 18, 16, 16, 15, 16, 15, 14, 14, 14, 13, 13,
    13, 12, 12, 11, 11, 10, 10, 8, 6, 4, 2, 1, 3, 5, 8, 9, 11, 11, 12, 13, 13, 14, 14, 14,
    15, 15, 16, 17, 18, 19, 18, 18, 18, 18, 20, 19, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 19,
    ],
    codewords: &[
    0xff7d6, 0xff7d7, 0xff7d8, 0xff7d9, 0xff7da, 0xff7db, 0xff7dc, 0xff7dd, 0xff7de, 0x7fbfe, 0x7ed7a, 0xff7df,
    0xff7f2, 0x3fdf4, 0xff7c, 0xfdae, 0x7ede, 0xff7e, 0x7ed2, 0x3f68, 0x3f6e, 0x3ffa, 0x1fb6, 0x1ffe,
    0x1fff, 0xfde, 0xff6, 0x7ec, 0x7fa, 0x3fc, 0x3fe, 0xfc, 0x3e, 0xe, 0x2, 0x0,
    0x6, 0x1e, 0xfe, 0x1fa, 0x7fe, 0x7ee, 0xfdf, 0x1ffc, 0x1fee, 0x3ffb, 0x3fde, 0x3f6a,
    0x7ed6, 0x7ed3, 0xfdbe, 0x1fb7e, 0x3f6bc, 0x7fbf8, 0x3f6be, 0x3f6bf, 0x3f6fe, 0x3f6ff, 0xff7f3, 0x7fbea,
    0xff7f4, 0xff7f5, 0xff7f6, 0xff7f7, 0xff7f8, 0xff7f9, 0xff7fa, 0xff7fb, 0xff7fe, 0xff7ff, 0x7ed7b,
    ],
};

pub static ASPX_HCB_ENV_LEVEL_30_F0: Codebook = Codebook {
    lengths: &[
    11, 13, 11, 11, 10, 10, 9, 8, 8, 7, 7, 7, 6, 6, 5, 5, 4, 4, 4, 4, 3, 3, 3, 3,
    4, 5, 6, 7, 9, 12, 14, 16, 17, 16, 17, 16,
    ],
    codewords: &[
    0x7fe, 0x1ffe, 0x1de, 0x1df, 0xee, 0x3fe, 0x1fe, 0x3a, 0xfe, 0x1c, 0x1e, 0x7e,
    0x2e, 0x3e, 0x6, 0x16, 0x2, 0xa, 0xc, 0xe, 0x2, 0x3, 0x4, 0x0,
    0xd, 0x1e, 0x2f, 0x1f, 0x76, 0xffe, 0x3ffe, 0xfffc, 0x1fffe, 0xfffe, 0x1ffff, 0xfffd,
    ],
};

pub static ASPX_HCB_NOISE_BALANCE_DF: Codebook = Codebook {
    lengths: &[
    13, 13, 13, 13, 13, 12, 13, 10, 9, 7, 4, 2, 1, 3, 5, 6, 9, 11, 13, 13, 12, 12, 12, 12,
    12,
    ],
    codewords: &[
    0x1fd8, 0x1fd9, 0x1fda, 0x1fdb, 0x1fdc, 0xfea, 0x1fdd, 0x3f8, 0x1fe, 0x7e, 0xe, 0x2,
    0x0, 0x6, 0x1e, 0x3e, 0x1ff, 0x7f4, 0x1fde, 0x1fdf, 0xfe4, 0xfe5, 0xfe6, 0xfe7,
    0xfeb,
    ],
};

pub static ASPX_HCB_NOISE_BALANCE_DT: Codebook = Codebook {
    lengths: &[
    11, 11, 11, 11, 11, 11, 11, 10, 10, 6, 4, 3, 1, 2, 5, 8, 9, 11, 10, 11, 11, 11, 11, 11,
    11,
    ],
    codewords: &[
    0x7e6, 0x7e7, 0x7e8, 0x7e9, 0x7ea, 0x7eb, 0x7ec, 0x3fe, 0x3ff, 0x3e, 0xe, 0x6,
    0x0, 0x2, 0x1e, 0xfe, 0x1f8, 0x7ed, 0x3f2, 0x7ee, 0x7ef, 0x7f8, 0x7f9, 0x7fa,
    0x7fb,
    ],
};

pub static ASPX_HCB_NOISE_BALANCE_F0: Codebook = Codebook {
    lengths: &[
    10, 10, 9, 7, 5, 3, 2, 1, 4, 6, 9, 10, 10,
    ],
    codewords: &[
    0x3fa, 0x3fe, 0x1fc, 0x7e, 0x1e, 0x6, 0x2, 0x0, 0xe, 0x3e, 0x1fe, 0x3ff,
    0x3fb,
    ],
};

pub static ASPX_HCB_NOISE_LEVEL_DF: Codebook = Codebook {
    lengths: &[
    17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 13, 14, 16, 16, 15, 15, 16, 15, 14, 14, 14, 14, 13, 11,
    10, 9, 8, 7, 3, 1, 2, 4, 5, 7, 8, 9, 10, 12, 13, 13, 14, 16, 17, 17, 17, 17, 17, 17,
    16, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17,
    ],
    codewords: &[
    0x1f19e, 0x1f19f, 0x1f1dc, 0x1f1dd, 0x1f1de, 0x1f1df, 0x1f1e8, 0x1f1e9, 0x1f1ea, 0x1f1eb, 0x1f1a, 0x3e7e,
    0xf9fe, 0xf8f6, 0x7c76, 0x7c66, 0xf8f7, 0x7c7c, 0x3e3c, 0x3e36, 0x3e37, 0x3e3a, 0x1f3e, 0x7ce,
    0x3e6, 0x1f0, 0xfa, 0x7e, 0x6, 0x0, 0x2, 0xe, 0x1e, 0x7f, 0xfb, 0x1f2,
    0x3e2, 0xf9e, 0x1f1c, 0x1f18, 0x3e32, 0xf8fa, 0x1f3fe, 0x1f1f6, 0x1f1f7, 0x1f1f8, 0x1f1f9, 0x1f1fa,
    0xf8ce, 0x1f1fb, 0x1f1fc, 0x1f1fd, 0x1f1fe, 0x1f1ff, 0x1f3f8, 0x1f3f9, 0x1f3fa, 0x1f3fb, 0x1f3ff,
    ],
};

pub static ASPX_HCB_NOISE_LEVEL_DT: Codebook = Codebook {
    lengths: &[
    16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 14, 16, 13,
    11, 8, 6, 4, 3, 1, 2, 5, 7, 10, 10, 12, 13, 13, 14, 15, 14, 15, 15, 16, 16, 16, 16, 16,
    16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16,
    ],
    codewords: &[
    0xff70, 0xff71, 0xff72, 0xff73, 0xff74, 0xff75, 0xff76, 0xff77, 0xff78, 0xff79, 0xff7a, 0xff7b,
    0xff7c, 0xff7d, 0xff7e, 0xff7f, 0xffe0, 0xffe1, 0xffe2, 0xffe3, 0xffe6, 0x3fd6, 0xffe7, 0x1ffe,
    0x7fe, 0xfe, 0x3e, 0xe, 0x6, 0x0, 0x2, 0x1e, 0x7e, 0x3fe, 0x3fc, 0xff6,
    0x1fe8, 0x1fea, 0x3fd2, 0x7fae, 0x3fd3, 0x7ff2, 0x7faf, 0xffe8, 0xffe9, 0xffea, 0xffeb, 0xffec,
    0xffed, 0xffee, 0xffef, 0xfff8, 0xfff9, 0xfffa, 0xfffb, 0xfffc, 0xfffd, 0xfffe, 0xffff,
    ],
};

pub static ASPX_HCB_NOISE_LEVEL_F0: Codebook = Codebook {
    lengths: &[
    12, 12, 9, 7, 6, 6, 5, 1, 2, 3, 5, 7, 8, 8, 9, 10, 11, 12, 12, 12, 13, 13, 14, 12,
    14, 14, 11, 16, 16, 15,
    ],
    codewords: &[
    0xefe, 0xeff, 0x1fe, 0x76, 0x3a, 0x3e, 0x1e, 0x0, 0x2, 0x6, 0x1c, 0x7e,
    0xfe, 0xee, 0x1ff, 0x3bc, 0x77c, 0xefa, 0xef4, 0xef6, 0x1df6, 0x1dee, 0x3bee, 0xef5,
    0x3bde, 0x3bef, 0x77e, 0xef7e, 0xef7f, 0x77be,
    ],
};

pub static DE_HCB_ABS_0: Codebook = Codebook {
    lengths: &[
    3, 3, 4, 4, 5, 5, 5, 5, 4, 4, 3, 7, 7, 7, 8, 7, 7, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 7, 7, 8,
    ],
    codewords: &[
    0x6, 0x3, 0x9, 0x1, 0x1f, 0x1d, 0x15, 0x17, 0x0, 0x3, 0x2, 0x79,
    0x15, 0x10, 0x29, 0x78, 0x5a, 0x20, 0x3d, 0x39, 0x22, 0x21, 0x28, 0x2c,
    0x29, 0x38, 0x23, 0xb, 0x9, 0x5b, 0x11, 0x28,
    ],
};

pub static DE_HCB_ABS_1: Codebook = Codebook {
    lengths: &[
    9, 12, 12, 12, 12, 12, 12, 11, 10, 11, 11, 10, 10, 10, 10, 10, 10, 10, 9, 9, 9, 9, 8, 8,
    8, 7, 6, 6, 6, 5, 1, 4, 5, 5, 5, 5, 5, 5, 6, 6, 5, 6, 7, 6, 7, 8, 8, 9,
    9, 9, 9, 10, 10, 10, 11, 10, 11, 11, 12, 12, 10,
    ],
    codewords: &[
    0x15c, 0xaea, 0xaeb, 0xc56, 0xc78, 0xc79, 0xe51, 0x427, 0x210, 0x62a, 0x63d, 0x212,
    0x21d, 0x31d, 0x2bb, 0x314, 0x390, 0x395, 0x10b, 0x15f, 0x18b, 0x1cb, 0x86, 0xab,
    0xc6, 0x54, 0x20, 0x24, 0x38, 0x1e, 0x0, 0xd, 0x1f, 0x17, 0x16, 0x14,
    0x13, 0x11, 0x3b, 0x3a, 0x19, 0x25, 0x73, 0x30, 0x56, 0xc4, 0xaa, 0x1c9,
    0x15e, 0x10f, 0x10a, 0x391, 0x31c, 0x21c, 0x729, 0x211, 0x574, 0x426, 0xe50, 0xc57,
    0x31f,
    ],
};

pub static DE_HCB_DIFF_0: Codebook = Codebook {
    lengths: &[
    14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 13, 13, 13, 10, 10, 9,
    8, 8, 7, 6, 5, 4, 3, 1, 3, 5, 5, 6, 7, 7, 7, 8, 8, 8, 10, 10, 10, 11, 11, 11,
    11, 12, 12, 13, 13, 13, 14, 14, 14, 14, 14, 14, 14, 14, 13,
    ],
    codewords: &[
    0x2b0, 0x2b1, 0x2b4, 0x2b5, 0x2b7, 0x2b6, 0x2ba, 0x2bb, 0x2bc, 0x79d, 0x148d, 0x14b1,
    0x79e, 0x79f, 0x148f, 0x14b6, 0x14b4, 0x14b5, 0x14c, 0x159, 0x3cc, 0x2a, 0x14a, 0x3d,
    0xb, 0x57, 0x28, 0x3, 0x2, 0x4, 0x3, 0x1, 0x1, 0xb, 0x0, 0x6,
    0x2a, 0xe, 0x4, 0x53, 0x1f, 0x56, 0x149, 0x78, 0x28, 0x297, 0x290, 0xf2,
    0x52, 0x522, 0xa7, 0xa59, 0x3cd, 0x15c, 0x14b7, 0x14b0, 0x148e, 0x148c, 0x79c, 0x2bf,
    0x2bd, 0x2be, 0x14d,
    ],
};

pub static DE_HCB_DIFF_1: Codebook = Codebook {
    lengths: &[
    13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 12, 13, 13, 13,
    13, 13, 13, 12, 12, 12, 11, 12, 12, 12, 12, 12, 12, 12, 11, 11, 11, 11, 11, 10, 10, 10, 10, 9,
    9, 9, 8, 8, 7, 7, 7, 6, 6, 5, 4, 3, 1, 4, 5, 5, 6, 6, 7, 7, 8, 8, 8, 9,
    9, 9, 10, 10, 11, 11, 11, 11, 11, 12, 11, 12, 12, 12, 12, 12, 12, 12, 12, 13, 13, 13, 13, 13,
    13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13,
    13,
    ],
    codewords: &[
    0x1cf0, 0x1cbd, 0x1cbc, 0x1ccc, 0x1cb8, 0x1ccb, 0x1ccf, 0x1cc6, 0x1cca, 0x1cc9, 0x1cc8, 0x1cce,
    0x1cc5, 0x1cd8, 0x1cc4, 0x1cdf, 0x1cd1, 0x1cd5, 0x1ce2, 0x1ce3, 0xc21, 0x1cfc, 0x1cf1, 0x1cf2,
    0x1cf6, 0x1cf4, 0x1cf7, 0xc20, 0xc24, 0xc25, 0x729, 0xd1e, 0xd1f, 0xd59, 0xd5b, 0xd5f,
    0xe6d, 0xe77, 0x61b, 0x613, 0x6a8, 0x6ae, 0x730, 0x30a, 0x30c, 0x355, 0x396, 0x16c,
    0x1a2, 0x1bb, 0xd0, 0xdc, 0x5a, 0x69, 0x6f, 0x31, 0x38, 0x19, 0xa, 0x4,
    0x0, 0xf, 0x1d, 0x17, 0x36, 0x2c, 0x6b, 0x60, 0xe4, 0xd4, 0xb7, 0x1ba,
    0x187, 0x16d, 0x395, 0x30b, 0x73e, 0x728, 0x6a9, 0x68d, 0x68e, 0xe7f, 0x611, 0xd5e,
    0xd5a, 0xd58, 0xd19, 0xd18, 0xc34, 0xc35, 0xe74, 0x1cfd, 0x1cf3, 0x1cf5, 0x1cec, 0x1ced,
    0x1cea, 0x1ce7, 0x1ce5, 0x1ce6, 0x1ce4, 0x1cdd, 0x1ce1, 0x1ce0, 0x1cd4, 0x1cde, 0x1cd7, 0x1cdc,
    0x1cd2, 0x1cd6, 0x1ccd, 0x1cd3, 0x1cd9, 0x1cbf, 0x1cbe, 0x1cd0, 0x1cbb, 0x1cba, 0x1cb9, 0x1cc7,
    0x1ceb,
    ],
};

pub static DRC_HCB: Codebook = Codebook {
    lengths: &[
    12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12,
    12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12,
    12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12,
    12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12,
    12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 11, 11, 10, 10, 8, 9, 9, 9, 9, 9, 8, 8,
    7, 7, 6, 6, 5, 4, 3, 2, 3, 4, 4, 5, 5, 6, 6, 7, 7, 7, 7, 7, 7, 7, 7, 12,
    12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12,
    12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12,
    12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12,
    12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12,
    12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12,
    ],
    codewords: &[
    0xe12, 0xe11, 0xe10, 0xe0f, 0xe0e, 0xe0d, 0xe0c, 0xe0b, 0xe0a, 0xe09, 0xe08, 0xe07,
    0xe06, 0xe05, 0xe04, 0xe03, 0xe02, 0xe01, 0xe00, 0x53f, 0x53e, 0x53d, 0x53c, 0x53b,
    0x53a, 0x539, 0x538, 0x537, 0x536, 0x535, 0x534, 0x533, 0x532, 0x531, 0x530, 0x52f,
    0x52e, 0x52d, 0x52c, 0x52b, 0x52a, 0x529, 0x528, 0x527, 0x526, 0x525, 0x524, 0x523,
    0x522, 0x521, 0x520, 0x51f, 0x51e, 0x51d, 0x51c, 0x51b, 0x51a, 0x519, 0x518, 0x517,
    0x516, 0x515, 0x514, 0x513, 0x512, 0x511, 0x510, 0x50f, 0x50e, 0x50d, 0x50c, 0x50b,
    0x50a, 0x509, 0x508, 0x507, 0x506, 0x505, 0x504, 0x503, 0x502, 0x501, 0x500, 0x4ff,
    0x4fe, 0x4fd, 0x4fc, 0x4fb, 0x4fa, 0x4f9, 0x4f8, 0x4f7, 0x4f6, 0x4f5, 0x4f4, 0x4f3,
    0x4f2, 0x4f1, 0x4f0, 0x4ef, 0x4ee, 0x4ed, 0x4ec, 0x4eb, 0x4ea, 0x4e9, 0x4e8, 0xe13,
    0xba, 0xbb, 0x5c, 0x385, 0x16, 0x1c3, 0x2f, 0x54, 0x55, 0x56, 0x22, 0x23,
    0xd, 0x9, 0xb, 0x39, 0x1d, 0xf, 0x3, 0x2, 0x6, 0x3, 0x0, 0xb,
    0x8, 0x15, 0x7, 0x71, 0x14, 0x12, 0x13, 0xc, 0xa, 0x10, 0x8, 0x4e7,
    0x4e6, 0x4e5, 0x4e4, 0x4e3, 0x4e2, 0x4e1, 0x4e0, 0x4df, 0x4de, 0x4dd, 0x4dc, 0x4db,
    0x4da, 0x4d9, 0x4d8, 0x4d7, 0x4d6, 0x4d5, 0x4d4, 0x4d3, 0x4d2, 0x4d1, 0x4d0, 0x4cf,
    0x4ce, 0x4cd, 0x4cc, 0x4cb, 0x4ca, 0x4c9, 0x4c8, 0x4c7, 0x4c6, 0x4c5, 0x4c4, 0x4c3,
    0x4c2, 0x4c1, 0x4c0, 0x4bf, 0x4be, 0x4bd, 0x4bc, 0x4bb, 0x4ba, 0x4b9, 0x4b8, 0x4b7,
    0x4b6, 0x4b5, 0x4b4, 0x4b3, 0x4b2, 0x4b1, 0x4b0, 0x4af, 0x4ae, 0x4ad, 0x4ac, 0x4ab,
    0x4aa, 0x4a9, 0x4a8, 0x4a7, 0x4a6, 0x4a5, 0x4a4, 0x4a3, 0x4a2, 0x4a1, 0x4a0, 0x49f,
    0x49e, 0x49d, 0x49c, 0x49b, 0x49a, 0x499, 0x498, 0x497, 0x496, 0x495, 0x494, 0x493,
    0x492, 0x491, 0x490, 0x48f, 0x48e, 0x48d, 0x48c, 0x48b, 0x48a, 0x489, 0x488, 0x487,
    0x486, 0x485, 0x484, 0x483, 0x482, 0x481, 0x480, 0x2bf, 0x2be, 0x2bd, 0x2bc, 0x2bb,
    0x2ba, 0x2b9, 0x2b8,
    ],
};

pub fn codebook(name: &str) -> Option<&'static Codebook> {
    match name {
        "ACPL_HCB_ALPHA_COARSE_DF" => Some(&ACPL_HCB_ALPHA_COARSE_DF),
        "ACPL_HCB_ALPHA_COARSE_DT" => Some(&ACPL_HCB_ALPHA_COARSE_DT),
        "ACPL_HCB_ALPHA_COARSE_F0" => Some(&ACPL_HCB_ALPHA_COARSE_F0),
        "ACPL_HCB_ALPHA_FINE_DF" => Some(&ACPL_HCB_ALPHA_FINE_DF),
        "ACPL_HCB_ALPHA_FINE_DT" => Some(&ACPL_HCB_ALPHA_FINE_DT),
        "ACPL_HCB_ALPHA_FINE_F0" => Some(&ACPL_HCB_ALPHA_FINE_F0),
        "ACPL_HCB_BETA3_COARSE_DF" => Some(&ACPL_HCB_BETA3_COARSE_DF),
        "ACPL_HCB_BETA3_COARSE_DT" => Some(&ACPL_HCB_BETA3_COARSE_DT),
        "ACPL_HCB_BETA3_COARSE_F0" => Some(&ACPL_HCB_BETA3_COARSE_F0),
        "ACPL_HCB_BETA3_FINE_DF" => Some(&ACPL_HCB_BETA3_FINE_DF),
        "ACPL_HCB_BETA3_FINE_DT" => Some(&ACPL_HCB_BETA3_FINE_DT),
        "ACPL_HCB_BETA3_FINE_F0" => Some(&ACPL_HCB_BETA3_FINE_F0),
        "ACPL_HCB_BETA_COARSE_DF" => Some(&ACPL_HCB_BETA_COARSE_DF),
        "ACPL_HCB_BETA_COARSE_DT" => Some(&ACPL_HCB_BETA_COARSE_DT),
        "ACPL_HCB_BETA_COARSE_F0" => Some(&ACPL_HCB_BETA_COARSE_F0),
        "ACPL_HCB_BETA_FINE_DF" => Some(&ACPL_HCB_BETA_FINE_DF),
        "ACPL_HCB_BETA_FINE_DT" => Some(&ACPL_HCB_BETA_FINE_DT),
        "ACPL_HCB_BETA_FINE_F0" => Some(&ACPL_HCB_BETA_FINE_F0),
        "ACPL_HCB_GAMMA_COARSE_DF" => Some(&ACPL_HCB_GAMMA_COARSE_DF),
        "ACPL_HCB_GAMMA_COARSE_DT" => Some(&ACPL_HCB_GAMMA_COARSE_DT),
        "ACPL_HCB_GAMMA_COARSE_F0" => Some(&ACPL_HCB_GAMMA_COARSE_F0),
        "ACPL_HCB_GAMMA_FINE_DF" => Some(&ACPL_HCB_GAMMA_FINE_DF),
        "ACPL_HCB_GAMMA_FINE_DT" => Some(&ACPL_HCB_GAMMA_FINE_DT),
        "ACPL_HCB_GAMMA_FINE_F0" => Some(&ACPL_HCB_GAMMA_FINE_F0),
        "AJCC_HCB_DRY_COARSE_DF" => Some(&AJCC_HCB_DRY_COARSE_DF),
        "AJCC_HCB_DRY_COARSE_DT" => Some(&AJCC_HCB_DRY_COARSE_DT),
        "AJCC_HCB_DRY_COARSE_F0" => Some(&AJCC_HCB_DRY_COARSE_F0),
        "AJCC_HCB_DRY_FINE_DF" => Some(&AJCC_HCB_DRY_FINE_DF),
        "AJCC_HCB_DRY_FINE_DT" => Some(&AJCC_HCB_DRY_FINE_DT),
        "AJCC_HCB_DRY_FINE_F0" => Some(&AJCC_HCB_DRY_FINE_F0),
        "AJCC_HCB_WET_COARSE_DF" => Some(&AJCC_HCB_WET_COARSE_DF),
        "AJCC_HCB_WET_COARSE_DT" => Some(&AJCC_HCB_WET_COARSE_DT),
        "AJCC_HCB_WET_COARSE_F0" => Some(&AJCC_HCB_WET_COARSE_F0),
        "AJCC_HCB_WET_FINE_DF" => Some(&AJCC_HCB_WET_FINE_DF),
        "AJCC_HCB_WET_FINE_DT" => Some(&AJCC_HCB_WET_FINE_DT),
        "AJCC_HCB_WET_FINE_F0" => Some(&AJCC_HCB_WET_FINE_F0),
        "AJOC_HCB_DRY_COARSE_DF" => Some(&AJOC_HCB_DRY_COARSE_DF),
        "AJOC_HCB_DRY_COARSE_DT" => Some(&AJOC_HCB_DRY_COARSE_DT),
        "AJOC_HCB_DRY_COARSE_F0" => Some(&AJOC_HCB_DRY_COARSE_F0),
        "AJOC_HCB_DRY_FINE_DF" => Some(&AJOC_HCB_DRY_FINE_DF),
        "AJOC_HCB_DRY_FINE_DT" => Some(&AJOC_HCB_DRY_FINE_DT),
        "AJOC_HCB_DRY_FINE_F0" => Some(&AJOC_HCB_DRY_FINE_F0),
        "AJOC_HCB_WET_COARSE_DF" => Some(&AJOC_HCB_WET_COARSE_DF),
        "AJOC_HCB_WET_COARSE_DT" => Some(&AJOC_HCB_WET_COARSE_DT),
        "AJOC_HCB_WET_COARSE_F0" => Some(&AJOC_HCB_WET_COARSE_F0),
        "AJOC_HCB_WET_FINE_DF" => Some(&AJOC_HCB_WET_FINE_DF),
        "AJOC_HCB_WET_FINE_DT" => Some(&AJOC_HCB_WET_FINE_DT),
        "AJOC_HCB_WET_FINE_F0" => Some(&AJOC_HCB_WET_FINE_F0),
        "ASF_HCB_1" => Some(&ASF_HCB_1),
        "ASF_HCB_10" => Some(&ASF_HCB_10),
        "ASF_HCB_11" => Some(&ASF_HCB_11),
        "ASF_HCB_2" => Some(&ASF_HCB_2),
        "ASF_HCB_3" => Some(&ASF_HCB_3),
        "ASF_HCB_4" => Some(&ASF_HCB_4),
        "ASF_HCB_5" => Some(&ASF_HCB_5),
        "ASF_HCB_6" => Some(&ASF_HCB_6),
        "ASF_HCB_7" => Some(&ASF_HCB_7),
        "ASF_HCB_8" => Some(&ASF_HCB_8),
        "ASF_HCB_9" => Some(&ASF_HCB_9),
        "ASF_HCB_SCALEFAC" => Some(&ASF_HCB_SCALEFAC),
        "ASF_HCB_SNF" => Some(&ASF_HCB_SNF),
        "ASPX_HCB_ENV_BALANCE_15_DF" => Some(&ASPX_HCB_ENV_BALANCE_15_DF),
        "ASPX_HCB_ENV_BALANCE_15_DT" => Some(&ASPX_HCB_ENV_BALANCE_15_DT),
        "ASPX_HCB_ENV_BALANCE_15_F0" => Some(&ASPX_HCB_ENV_BALANCE_15_F0),
        "ASPX_HCB_ENV_BALANCE_30_DF" => Some(&ASPX_HCB_ENV_BALANCE_30_DF),
        "ASPX_HCB_ENV_BALANCE_30_DT" => Some(&ASPX_HCB_ENV_BALANCE_30_DT),
        "ASPX_HCB_ENV_BALANCE_30_F0" => Some(&ASPX_HCB_ENV_BALANCE_30_F0),
        "ASPX_HCB_ENV_LEVEL_15_DF" => Some(&ASPX_HCB_ENV_LEVEL_15_DF),
        "ASPX_HCB_ENV_LEVEL_15_DT" => Some(&ASPX_HCB_ENV_LEVEL_15_DT),
        "ASPX_HCB_ENV_LEVEL_15_F0" => Some(&ASPX_HCB_ENV_LEVEL_15_F0),
        "ASPX_HCB_ENV_LEVEL_30_DF" => Some(&ASPX_HCB_ENV_LEVEL_30_DF),
        "ASPX_HCB_ENV_LEVEL_30_DT" => Some(&ASPX_HCB_ENV_LEVEL_30_DT),
        "ASPX_HCB_ENV_LEVEL_30_F0" => Some(&ASPX_HCB_ENV_LEVEL_30_F0),
        "ASPX_HCB_NOISE_BALANCE_DF" => Some(&ASPX_HCB_NOISE_BALANCE_DF),
        "ASPX_HCB_NOISE_BALANCE_DT" => Some(&ASPX_HCB_NOISE_BALANCE_DT),
        "ASPX_HCB_NOISE_BALANCE_F0" => Some(&ASPX_HCB_NOISE_BALANCE_F0),
        "ASPX_HCB_NOISE_LEVEL_DF" => Some(&ASPX_HCB_NOISE_LEVEL_DF),
        "ASPX_HCB_NOISE_LEVEL_DT" => Some(&ASPX_HCB_NOISE_LEVEL_DT),
        "ASPX_HCB_NOISE_LEVEL_F0" => Some(&ASPX_HCB_NOISE_LEVEL_F0),
        "DE_HCB_ABS_0" => Some(&DE_HCB_ABS_0),
        "DE_HCB_ABS_1" => Some(&DE_HCB_ABS_1),
        "DE_HCB_DIFF_0" => Some(&DE_HCB_DIFF_0),
        "DE_HCB_DIFF_1" => Some(&DE_HCB_DIFF_1),
        "DRC_HCB" => Some(&DRC_HCB),
        _ => None,
    }
}

static SFB_OFFSETS_96: [u16; 25] = [
    0, 4, 8, 12, 16, 20, 28, 36, 44, 56, 68, 80, 96, 112, 128, 144,
    160, 176, 192, 224, 256, 288, 320, 352, 384,
];
static SFB_OFFSETS_120: [u16; 31] = [
    0, 4, 8, 12, 16, 20, 28, 36, 44, 56, 68, 80, 96, 112, 120, 128,
    144, 160, 176, 192, 208, 224, 240, 256, 288, 320, 352, 384, 416, 448, 480,
];
static SFB_OFFSETS_128: [u16; 31] = [
    0, 4, 8, 12, 16, 20, 28, 36, 44, 56, 68, 80, 96, 112, 128, 144,
    160, 176, 192, 208, 224, 240, 256, 288, 320, 352, 384, 416, 448, 480, 512,
];
static SFB_OFFSETS_192: [u16; 31] = [
    0, 4, 8, 12, 16, 20, 24, 28, 36, 44, 52, 64, 76, 92, 108, 128,
    148, 172, 192, 224, 256, 288, 320, 352, 384, 448, 512, 576, 640, 704, 768,
];
static SFB_OFFSETS_240: [u16; 37] = [
    0, 4, 8, 12, 16, 20, 24, 28, 36, 44, 52, 64, 76, 92, 108, 128,
    148, 172, 196, 224, 240, 256, 288, 320, 352, 384, 416, 448, 480, 512, 576, 640,
    704, 768, 832, 896, 960,
];
static SFB_OFFSETS_256: [u16; 37] = [
    0, 4, 8, 12, 16, 20, 24, 28, 36, 44, 52, 64, 76, 92, 108, 128,
    148, 172, 196, 224, 256, 288, 320, 352, 384, 416, 448, 480, 512, 576, 640, 704,
    768, 832, 896, 960, 1024,
];
static SFB_OFFSETS_384: [u16; 52] = [
    0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 44, 48, 52, 56, 60,
    68, 76, 84, 92, 100, 112, 124, 136, 148, 164, 184, 208, 236, 268, 300, 332,
    364, 384, 448, 512, 576, 640, 704, 768, 832, 896, 960, 1024, 1088, 1152, 1216, 1280,
    1344, 1408, 1472, 1536,
];
static SFB_OFFSETS_480: [u16; 60] = [
    0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 44, 48, 52, 56, 60,
    68, 76, 84, 92, 100, 112, 124, 136, 148, 164, 184, 208, 236, 268, 300, 332,
    364, 396, 428, 460, 480, 512, 576, 640, 704, 768, 832, 896, 960, 1024, 1088, 1152,
    1216, 1280, 1344, 1408, 1472, 1536, 1600, 1664, 1728, 1792, 1856, 1920,
];
static SFB_OFFSETS_512: [u16; 61] = [
    0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 44, 48, 52, 56, 60,
    68, 76, 84, 92, 100, 112, 124, 136, 148, 164, 184, 208, 236, 268, 300, 332,
    364, 396, 428, 460, 512, 576, 640, 704, 768, 832, 896, 960, 1024, 1088, 1152, 1216,
    1280, 1344, 1408, 1472, 1536, 1600, 1664, 1728, 1792, 1856, 1920, 1984, 2048,
];
static SFB_OFFSETS_768: [u16; 62] = [
    0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 48, 56, 64, 72, 80,
    88, 96, 108, 120, 132, 144, 160, 176, 196, 216, 240, 264, 292, 320, 352, 384,
    416, 448, 480, 512, 544, 576, 608, 640, 672, 704, 736, 768, 896, 1024, 1152, 1280,
    1408, 1536, 1664, 1792, 1920, 2048, 2176, 2304, 2432, 2560, 2688, 2816, 2944, 3072,
];
static SFB_OFFSETS_960: [u16; 73] = [
    0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 48, 56, 64, 72, 80,
    88, 96, 108, 120, 132, 144, 160, 176, 196, 216, 240, 264, 292, 320, 352, 384,
    416, 448, 480, 512, 544, 576, 608, 640, 672, 704, 736, 768, 800, 832, 864, 896,
    928, 960, 1024, 1152, 1280, 1408, 1536, 1664, 1792, 1920, 2048, 2176, 2304, 2432, 2560, 2688,
    2816, 2944, 3072, 3200, 3328, 3456, 3584, 3712, 3840,
];
static SFB_OFFSETS_1024: [u16; 74] = [
    0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 48, 56, 64, 72, 80,
    88, 96, 108, 120, 132, 144, 160, 176, 196, 216, 240, 264, 292, 320, 352, 384,
    416, 448, 480, 512, 544, 576, 608, 640, 672, 704, 736, 768, 800, 832, 864, 896,
    928, 1024, 1152, 1280, 1408, 1536, 1664, 1792, 1920, 2048, 2176, 2304, 2432, 2560, 2688, 2816,
    2944, 3072, 3200, 3328, 3456, 3584, 3712, 3840, 3968, 4096,
];
static SFB_OFFSETS_1536: [u16; 92] = [
    0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 44, 52, 60, 68, 76,
    84, 92, 100, 108, 116, 124, 136, 148, 160, 172, 188, 204, 220, 240, 260, 284,
    308, 336, 364, 396, 432, 468, 508, 552, 600, 652, 704, 768, 832, 896, 960, 1024,
    1088, 1152, 1216, 1280, 1344, 1408, 1472, 1536, 1664, 1792, 1920, 2048, 2176, 2304, 2432, 2560,
    2688, 2816, 2944, 3072, 3200, 3328, 3456, 3584, 3712, 3840, 3968, 4096, 4224, 4352, 4480, 4608,
    4736, 4864, 4992, 5120, 5248, 5376, 5504, 5632, 5760, 5888, 6016, 6144,
];
static SFB_OFFSETS_1920: [u16; 107] = [
    0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 44, 52, 60, 68, 76,
    84, 92, 100, 108, 116, 124, 136, 148, 160, 172, 188, 204, 220, 240, 260, 284,
    308, 336, 364, 396, 432, 468, 508, 552, 600, 652, 704, 768, 832, 896, 960, 1024,
    1088, 1152, 1216, 1280, 1344, 1408, 1472, 1536, 1600, 1664, 1728, 1792, 1856, 1920, 2048, 2176,
    2304, 2432, 2560, 2688, 2816, 2944, 3072, 3200, 3328, 3456, 3584, 3712, 3840, 3968, 4096, 4224,
    4352, 4480, 4608, 4736, 4864, 4992, 5120, 5248, 5376, 5504, 5632, 5760, 5888, 6016, 6144, 6272,
    6400, 6528, 6656, 6784, 6912, 7040, 7168, 7296, 7424, 7552, 7680,
];
static SFB_OFFSETS_2048: [u16; 112] = [
    0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 44, 52, 60, 68, 76,
    84, 92, 100, 108, 116, 124, 136, 148, 160, 172, 188, 204, 220, 240, 260, 284,
    308, 336, 364, 396, 432, 468, 508, 552, 600, 652, 704, 768, 832, 896, 960, 1024,
    1088, 1152, 1216, 1280, 1344, 1408, 1472, 1536, 1600, 1664, 1728, 1792, 1856, 1920, 1984, 2048,
    2176, 2304, 2432, 2560, 2688, 2816, 2944, 3072, 3200, 3328, 3456, 3584, 3712, 3840, 3968, 4096,
    4224, 4352, 4480, 4608, 4736, 4864, 4992, 5120, 5248, 5376, 5504, 5632, 5760, 5888, 6016, 6144,
    6272, 6400, 6528, 6656, 6784, 6912, 7040, 7168, 7296, 7424, 7552, 7680, 7808, 7936, 8064, 8192,
];

pub fn sfb_offsets(n: usize) -> Option<&'static [u16]> {
    match n {
        96 => Some(&SFB_OFFSETS_96),
        120 => Some(&SFB_OFFSETS_120),
        128 => Some(&SFB_OFFSETS_128),
        192 => Some(&SFB_OFFSETS_192),
        240 => Some(&SFB_OFFSETS_240),
        256 => Some(&SFB_OFFSETS_256),
        384 => Some(&SFB_OFFSETS_384),
        480 => Some(&SFB_OFFSETS_480),
        512 => Some(&SFB_OFFSETS_512),
        768 => Some(&SFB_OFFSETS_768),
        960 => Some(&SFB_OFFSETS_960),
        1024 => Some(&SFB_OFFSETS_1024),
        1536 => Some(&SFB_OFFSETS_1536),
        1920 => Some(&SFB_OFFSETS_1920),
        2048 => Some(&SFB_OFFSETS_2048),
        _ => None,
    }
}

static SFB_SIDE_192_96: [u8; 8] = [
    0, 1, 1, 2, 2, 3, 3, 4,
];
static SFB_SIDE_240_120: [u8; 16] = [
    0, 1, 1, 2, 2, 3, 3, 4, 5, 6, 6, 7, 8, 9, 9, 10,
];
static SFB_SIDE_256_128: [u8; 16] = [
    0, 1, 1, 2, 2, 3, 3, 4, 5, 6, 6, 7, 8, 9, 9, 10,
];
static SFB_SIDE_384_96: [u8; 16] = [
    0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4,
];
static SFB_SIDE_384_192: [u8; 16] = [
    0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8,
];
static SFB_SIDE_480_120: [u8; 32] = [
    0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 5, 5, 6, 6, 6, 6, 7, 7,
    8, 8, 9, 9, 10, 10, 11, 12,
];
static SFB_SIDE_480_240: [u8; 32] = [
    0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12,
    12, 13, 13, 14, 15, 16, 17, 17,
];
static SFB_SIDE_512_128: [u8; 32] = [
    0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 5, 5, 6, 6, 6, 6, 7, 7,
    8, 8, 9, 9, 10, 10, 11, 12,
];
static SFB_SIDE_512_256: [u8; 32] = [
    0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12,
    12, 13, 13, 14, 15, 16, 17, 17,
];
static SFB_SIDE_768_96: [u8; 32] = [
    0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 5, 5, 5, 6,
    6, 6, 7, 7, 8, 8, 8, 9,
];
static SFB_SIDE_768_192: [u8; 32] = [
    0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 4, 4, 5, 5, 6, 6, 7, 8, 8, 8, 9, 9,
    10, 11, 11, 12, 12, 13, 13, 14,
];
static SFB_SIDE_768_384: [u8; 32] = [
    0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 7, 8, 9, 10, 11, 12, 14, 15, 16, 17, 18, 19,
    20, 21, 22, 23, 24, 25, 26, 27,
];
static SFB_SIDE_960_120: [u8; 32] = [
    0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 5, 5, 5, 6,
    6, 6, 7, 7, 8, 8, 8, 9,
];
static SFB_SIDE_960_240: [u8; 32] = [
    0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 4, 4, 5, 5, 6, 6, 7, 8, 8, 8, 9, 9,
    10, 11, 11, 12, 12, 13, 13, 14,
];
static SFB_SIDE_960_480: [u8; 32] = [
    0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 7, 8, 9, 10, 11, 12, 14, 15, 16, 17, 18, 19,
    20, 21, 22, 23, 24, 25, 26, 27,
];
static SFB_SIDE_1024_128: [u8; 32] = [
    0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 5, 5, 5, 6,
    6, 6, 7, 7, 8, 8, 8, 9,
];
static SFB_SIDE_1024_256: [u8; 32] = [
    0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 4, 4, 5, 5, 6, 6, 7, 8, 8, 8, 9, 9,
    10, 11, 11, 12, 12, 13, 13, 14,
];
static SFB_SIDE_1024_512: [u8; 32] = [
    0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 7, 8, 9, 10, 11, 12, 14, 15, 16, 17, 18, 19,
    20, 21, 22, 23, 24, 25, 26, 27,
];
static SFB_SIDE_1536_96: [u8; 32] = [
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3,
    3, 3, 3, 4, 4, 4, 5, 5,
];
static SFB_SIDE_1536_192: [u8; 32] = [
    0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5,
    5, 6, 6, 7, 7, 8, 8, 8,
];
static SFB_SIDE_1536_384: [u8; 32] = [
    0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 10,
    10, 11, 12, 13, 14, 15, 16, 17,
];
static SFB_SIDE_1536_768: [u8; 32] = [
    0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 7, 8, 9, 10, 11, 11, 12, 12, 13, 13, 14, 15,
    15, 16, 17, 18, 19, 19, 20, 21,
];
static SFB_SIDE_1920_120: [u8; 32] = [
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3,
    3, 3, 3, 4, 4, 4, 5, 5,
];
static SFB_SIDE_1920_240: [u8; 32] = [
    0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5,
    5, 6, 6, 7, 7, 8, 8, 8,
];
static SFB_SIDE_1920_480: [u8; 32] = [
    0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 10,
    10, 11, 12, 13, 14, 15, 16, 17,
];
static SFB_SIDE_1920_960: [u8; 32] = [
    0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 7, 8, 9, 10, 11, 11, 12, 12, 13, 13, 14, 15,
    15, 16, 17, 18, 19, 19, 20, 21,
];
static SFB_SIDE_2048_128: [u8; 32] = [
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3,
    3, 3, 3, 4, 4, 4, 5, 5,
];
static SFB_SIDE_2048_256: [u8; 32] = [
    0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5,
    5, 6, 6, 7, 7, 8, 8, 8,
];
static SFB_SIDE_2048_512: [u8; 32] = [
    0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 10,
    10, 11, 12, 13, 14, 15, 16, 17,
];
static SFB_SIDE_2048_1024: [u8; 32] = [
    0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 7, 8, 9, 10, 11, 11, 12, 12, 13, 13, 14, 15,
    15, 16, 17, 18, 19, 19, 20, 21,
];

pub fn sfb_side(largest: usize, n: usize) -> Option<&'static [u8]> {
    match (largest, n) {
        (192, 96) => Some(&SFB_SIDE_192_96),
        (240, 120) => Some(&SFB_SIDE_240_120),
        (256, 128) => Some(&SFB_SIDE_256_128),
        (384, 96) => Some(&SFB_SIDE_384_96),
        (384, 192) => Some(&SFB_SIDE_384_192),
        (480, 120) => Some(&SFB_SIDE_480_120),
        (480, 240) => Some(&SFB_SIDE_480_240),
        (512, 128) => Some(&SFB_SIDE_512_128),
        (512, 256) => Some(&SFB_SIDE_512_256),
        (768, 96) => Some(&SFB_SIDE_768_96),
        (768, 192) => Some(&SFB_SIDE_768_192),
        (768, 384) => Some(&SFB_SIDE_768_384),
        (960, 120) => Some(&SFB_SIDE_960_120),
        (960, 240) => Some(&SFB_SIDE_960_240),
        (960, 480) => Some(&SFB_SIDE_960_480),
        (1024, 128) => Some(&SFB_SIDE_1024_128),
        (1024, 256) => Some(&SFB_SIDE_1024_256),
        (1024, 512) => Some(&SFB_SIDE_1024_512),
        (1536, 96) => Some(&SFB_SIDE_1536_96),
        (1536, 192) => Some(&SFB_SIDE_1536_192),
        (1536, 384) => Some(&SFB_SIDE_1536_384),
        (1536, 768) => Some(&SFB_SIDE_1536_768),
        (1920, 120) => Some(&SFB_SIDE_1920_120),
        (1920, 240) => Some(&SFB_SIDE_1920_240),
        (1920, 480) => Some(&SFB_SIDE_1920_480),
        (1920, 960) => Some(&SFB_SIDE_1920_960),
        (2048, 128) => Some(&SFB_SIDE_2048_128),
        (2048, 256) => Some(&SFB_SIDE_2048_256),
        (2048, 512) => Some(&SFB_SIDE_2048_512),
        (2048, 1024) => Some(&SFB_SIDE_2048_1024),
        _ => None,
    }
}

pub static RANDOM_NOISE_TABLE: [f64; 256] = [
    0.64312, 0.970001, -0.340081, -0.64312, 0.88972, -0.61233, 0.332226, 0.038815,
    -0.248121, 0.211033, 0.987555, 0.152782, -0.30889, -0.109753, -0.218404, 0.138387,
    1.618787, -1.237989, -0.553833, 1.381896, 0.181761, -0.686079, -0.152782, 0.060027,
    -0.6972, -0.780194, 1.208686, 0.017636, -0.755469, 0.708487, 0.324411, -0.045881,
    -0.83258, -0.095503, -1.718228, 0.920482, 0.109753, -0.731594, 0.572915, -1.381896,
    -0.181761, 0.421076, -0.278261, -0.743431, 0.031753, -0.63273, -0.987555, 0.263134,
    -0.421076, -0.489838, 0.145577, 0.686079, 0.301183, 0.429454, 1.303538, 0.045881,
    1.269466, 0.653644, -1.428033, 0.60231, 1.718228, 0.664308, 0.340081, 0.124046,
    0.6972, -0.102623, 0.248121, -0.233213, -0.347976, -0.29351, -1.155382, 0.355913,
    0.446387, 0.379985, -0.463567, 0.404484, -0.936505, 0.904897, -0.819081, 0.08129,
    0.525906, -0.664308, 0.498739, -1.480806, 0.553833, 1.237989, 0.024693, 0.003527,
    -0.622469, 1.024513, -0.379985, 1.005707, 0.225797, -0.525906, 0.860482, 0.76772,
    -0.572915, -0.355913, 0.088392, 0.347976, -1.044036, -0.363892, 1.54284, -1.34076,
    -0.874923, 1.130919, -0.301183, -1.269466, -0.76772, -0.263134, 0.102623, 0.13121,
    -0.017636, -0.138387, -0.388102, 1.064347, 0.592404, 0.233213, 0.052951, 0.489838,
    -0.952999, -0.067108, 0.846374, 0.51677, 0.563325, -0.792903, -0.324411, 0.29351,
    -0.13121, -1.024513, -0.024693, 1.428033, -0.429454, 0.074195, 0.454945, -0.412753,
    -0.970001, -0.211033, 0.010581, -0.332226, 1.480806, -1.181236, -0.003527, -0.904897,
    0.743431, -0.582607, -0.446387, 0.622469, 0.255614, 1.181236, -0.371916, -0.255614,
    -0.052951, 0.544434, -0.145577, -0.08129, 0.83258, -0.805861, 0.805861, -1.303538,
    0.755469, -0.544434, -0.316632, 0.463567, -0.507715, 1.085529, -0.437891, 0.874923,
    0.819081, -0.535126, -0.708487, -0.203685, 0.371916, -1.005707, 0.116894, 0.792903,
    0.30889, 0.719949, -0.167238, 0.936505, 0.507715, -0.160002, -0.653644, -0.010581,
    0.203685, 0.396268, -0.481011, 0.582607, -1.107681, -0.124046, -0.196357, -1.064347,
    0.174491, 0.952999, 0.167238, 0.437891, 0.780194, 1.107681, 0.61233, -0.038815,
    0.316632, 0.731594, 0.278261, -1.54284, 0.412753, -0.240654, 0.63273, 0.363892,
    -0.472255, -0.174491, -0.719949, 0.196357, 0.28587, -1.618787, 0.18905, -0.074195,
    1.86707, -0.225797, 0.240654, -0.18905, 0.472255, 0.095503, -0.860482, -0.270683,
    -0.51677, 0.388102, -0.60231, -0.28587, -0.88972, -1.86707, 1.34076, -0.060027,
    0.218404, -0.592404, -1.208686, 0.675117, -0.116894, 1.155382, -0.088392, 0.535126,
    -0.675117, 0.160002, -0.563325, -0.031753, -0.920482, -0.498739, -0.404484, -1.130919,
    1.044036, -0.396268, -0.846374, -0.454945, 0.481011, -1.085529, 0.067108, 0.270683,
];

pub static ASPX_NOISE: [(f64, f64); 512] = [
    (-0.782083, -0.623174), (0.705088, -0.70912), (0.268786, -0.9632), (0.689305, -0.724471),
    (-0.0430946, -0.999071), (-0.998077, -0.0619922), (0.867875, -0.496783), (0.814907, -0.579591),
    (0.880168, 0.474663), (-0.39062, -0.920552), (0.0434465, 0.999056), (0.610173, -0.792268),
    (-0.942195, 0.335065), (-0.911161, 0.412052), (-0.999084, -0.0427925), (-0.811795, 0.583942),
    (0.836524, 0.54793), (0.475443, 0.879747), (0.961597, -0.274466), (0.820146, 0.572155),
    (-0.498318, -0.866994), (0.825586, -0.564277), (0.999973, 0.00730176), (0.895923, 0.44421),
    (-0.80517, -0.593044), (0.277753, 0.960652), (-0.999688, 0.0249582), (-0.802608, -0.596507),
    (0.936737, -0.350035), (-0.997477, -0.0709933), (-0.674713, -0.73808), (-0.957686, -0.287817),
    (0.983532, 0.180732), (0.634759, -0.77271), (-0.150723, -0.988576), (-0.113979, 0.993483),
    (-0.993827, -0.110942), (0.781536, 0.62386), (-0.615104, -0.788446), (-0.586834, 0.809707),
    (-0.226253, 0.974069), (0.949896, -0.312565), (0.830084, 0.557638), (0.60415, 0.796871),
    (0.655412, 0.755272), (-0.916524, -0.399979), (0.763757, -0.645503), (0.822298, 0.569058),
    (-0.0283442, -0.999598), (-0.644058, -0.764977), (-0.390833, -0.920462), (-0.984217, -0.176967),
    (0.719987, 0.693987), (0.999088, -0.042707), (0.754487, 0.656315), (0.498312, 0.866998),
    (0.999389, 0.0349481), (-0.811845, -0.583873), (0.532015, -0.846735), (-0.463781, 0.88595),
    (-0.819481, -0.573106), (0.637789, 0.770211), (0.814253, -0.58051), (0.180489, -0.983577),
    (0.988691, -0.149968), (0.606847, -0.794819), (-0.061914, 0.998081), (0.627066, 0.778966),
    (-0.544097, -0.839023), (-0.859249, -0.511558), (-0.91818, 0.396163), (-0.165942, 0.986136),
    (0.992322, 0.123685), (0.758555, 0.651609), (0.985543, -0.169427), (-0.154971, -0.987919),
    (0.245336, 0.969438), (-0.522038, -0.852922), (-0.33027, -0.943886), (0.998067, -0.0621421),
    (0.516758, 0.856132), (0.843123, -0.537721), (0.44306, 0.896492), (-0.814913, 0.579584),
    (-0.336464, -0.941696), (0.732896, 0.68034), (0.201774, -0.979432), (0.741954, 0.670451),
    (-0.469083, 0.883154), (0.867784, 0.496941), (0.494202, -0.869347), (0.9367, -0.350134),
    (0.906328, 0.422575), (0.764111, 0.645085), (0.631052, -0.77574), (-0.0498248, -0.998758),
    (0.974691, -0.223555), (0.361405, -0.932409), (-0.748625, 0.662994), (0.811839, -0.583881),
    (0.303931, 0.952694), (-0.992668, -0.120876), (-0.996672, -0.0815161), (-0.324622, 0.945844),
    (-0.0246385, -0.999696), (-0.588361, 0.808598), (-0.49898, 0.866613), (0.924578, -0.380994),
    (-0.755619, 0.655011), (0.92214, -0.386857), (0.818638, 0.57431), (-0.920451, 0.390857),
    (0.0380205, -0.999277), (0.446606, -0.894731), (0.722557, -0.691312), (0.762113, -0.647444),
    (-0.256731, -0.966483), (0.471301, -0.881972), (-0.530869, -0.847454), (-0.749876, 0.661579),
    (-0.593767, -0.804637), (-0.834805, 0.550545), (0.748843, 0.662747), (-0.70794, 0.706273),
    (-0.0503228, -0.998733), (0.402884, -0.915251), (0.0945791, -0.995517), (-0.390889, -0.920438),
    (-0.0994705, -0.995041), (-0.767207, -0.641399), (-0.563485, -0.826126), (-0.521859, 0.853032),
    (0.503637, 0.863915), (-0.739851, -0.672771), (0.442624, -0.896707), (-0.997671, 0.0682121),
    (0.776117, -0.630589), (-0.964601, -0.263713), (-0.656053, 0.754715), (-0.865578, 0.500775),
    (-0.586255, -0.810127), (0.0425286, -0.999095), (0.656339, 0.754466), (-0.341906, 0.939734),
    (-0.605904, 0.795538), (-0.658238, -0.752809), (-0.652856, 0.757482), (-0.994554, 0.104219),
    (0.176725, -0.98426), (-0.231945, 0.972729), (-0.997717, -0.0675303), (-0.997805, 0.0662198),
    (0.661155, 0.75025), (0.999669, -0.02573), (-0.946982, -0.321287), (-0.587897, 0.808936),
    (0.957862, -0.28723), (0.613392, -0.789779), (-0.956489, 0.291768), (0.169829, -0.985474),
    (-0.943551, -0.331226), (0.416834, 0.908983), (0.684727, -0.7288), (0.952329, 0.305074),
    (-0.328392, 0.944541), (0.943344, 0.331816), (0.650872, 0.759187), (-0.59941, -0.800442),
    (-0.768448, -0.639912), (0.539894, 0.841733), (0.606048, -0.795428), (0.403588, 0.914941),
    (-0.838111, 0.545499), (0.976157, -0.217066), (-0.995495, -0.0948107), (-0.943792, 0.330539),
    (-0.990415, 0.138123), (0.281355, 0.959604), (0.371208, 0.92855), (-0.4711, 0.88208),
    (-0.999459, -0.0328787), (-0.988179, 0.153305), (-0.843124, -0.537718), (0.997108, 0.0759971),
    (-0.268201, -0.963363), (0.0457651, 0.998952), (-0.983762, 0.179479), (-0.439728, -0.898131),
    (0.162945, -0.986635), (-0.055868, 0.998438), (-0.384381, 0.923174), (0.744138, -0.668026),
    (-0.0706482, -0.997501), (0.831219, 0.555945), (0.711624, 0.70256), (0.161772, -0.986828),
    (0.8387, 0.544593), (-0.418108, 0.908397), (0.412208, 0.91109), (-0.986564, 0.163375),
    (0.925222, 0.379427), (0.411092, -0.911594), (-0.103885, -0.994589), (-0.923358, -0.383941),
    (-0.761339, 0.648355), (-0.887774, -0.46028), (0.755699, -0.654919), (0.597832, -0.801622),
    (0.542946, -0.839768), (0.999842, -0.0177525), (-0.954864, 0.297044), (-0.999961, -0.00886517),
    (0.929688, -0.368347), (0.699205, 0.714921), (-0.894118, 0.447831), (-0.903496, 0.428597),
    (0.362774, -0.931877), (0.850158, -0.526527), (0.523993, -0.851722), (-0.735767, 0.677235),
    (-0.999673, -0.02558), (-0.954685, 0.297619), (-0.195193, -0.980765), (0.0672577, -0.997736),
    (-0.659842, -0.751405), (0.779366, 0.626569), (0.751561, -0.659663), (-0.34458, -0.938757),
    (0.316872, 0.948468), (0.953601, 0.301074), (0.363243, 0.931694), (-0.537299, -0.843392),
    (0.996839, 0.0794518), (0.950462, -0.310842), (0.224973, 0.974365), (-0.772894, -0.634535),
    (-0.94373, -0.330718), (0.930603, 0.36603), (0.994059, 0.10884), (0.845518, -0.533947),
    (0.988122, -0.15367), (0.880641, -0.473785), (0.783488, -0.621407), (0.72854, 0.685004),
    (-0.772294, 0.635265), (0.662444, 0.749111), (-0.0649291, 0.99789), (-0.285125, 0.95849),
    (-0.673637, -0.739062), (-0.394791, -0.918771), (-0.938677, -0.344798), (-0.708925, -0.705284),
    (0.31416, 0.94937), (-0.113645, 0.993521), (-0.296446, 0.95505), (0.670712, -0.741718),
    (-0.605825, 0.795598), (0.996229, 0.0867643), (0.686613, -0.727023), (-0.740136, 0.672457),
    (0.876977, 0.480532), (-0.561046, 0.827785), (0.414562, -0.910021), (-0.645953, -0.763377),
    (0.802263, 0.596971), (-0.854981, 0.51866), (-0.769916, 0.638145), (0.648047, 0.7616),
    (-0.773406, 0.633911), (-0.252579, -0.967576), (0.962561, -0.271064), (0.959193, -0.282751),
    (0.727508, 0.686099), (-0.667916, 0.744236), (-0.599333, 0.8005), (0.622504, 0.782616),
    (0.375433, -0.92685), (-0.9972, 0.0747742), (-0.879355, 0.476166), (0.409574, 0.912277),
    (0.747405, -0.664369), (-0.940177, -0.340687), (0.562532, 0.826776), (-0.929015, 0.370041),
    (0.0978642, 0.9952), (0.916896, -0.399125), (-0.608038, -0.793908), (-0.845653, -0.533732),
    (-0.455945, -0.890008), (0.923484, -0.383637), (0.354901, 0.934904), (0.319134, -0.94771),
    (0.769603, -0.638523), (-0.899207, 0.437524), (0.666669, -0.745354), (0.142655, -0.989772),
    (-0.892, -0.452035), (-0.999915, -0.0130018), (-0.87033, 0.492469), (0.156511, 0.987676),
    (-0.146752, 0.989173), (-0.809057, 0.58773), (-0.497325, -0.867565), (-0.258455, -0.966023),
    (-0.863292, -0.504705), (-0.976343, -0.216225), (-0.257626, -0.966245), (0.809568, -0.587027),
    (0.582491, 0.812837), (-0.997088, -0.0762565), (-0.878262, 0.47818), (-0.165343, -0.986236),
    (0.0455161, 0.998964), (-0.570664, 0.821184), (0.658564, 0.752525), (0.319839, -0.947472),
    (-0.643905, -0.765106), (-0.590256, 0.807216), (0.512137, 0.858904), (-0.998558, 0.0536785),
    (0.373964, -0.927443), (0.633108, -0.774063), (-0.968108, 0.250531), (0.787337, 0.616523),
    (0.698247, -0.715857), (0.98937, 0.145419), (0.582241, -0.813016), (0.359617, 0.9331),
    (-0.758164, -0.652064), (0.635102, 0.772429), (-0.0254028, -0.999677), (0.266382, -0.963868),
    (0.660974, -0.750409), (0.585176, -0.810906), (-0.98243, 0.186631), (0.777252, -0.629189),
    (-0.0267382, 0.999642), (-0.95591, 0.293661), (0.70368, -0.710517), (0.732467, -0.680803),
    (0.854099, -0.52011), (0.536151, 0.844122), (-0.00781503, 0.999969), (-0.534447, 0.845202),
    (0.297782, 0.954634), (0.905724, 0.423868), (0.115617, -0.993294), (-0.993408, -0.114636),
    (0.156977, 0.987602), (0.639408, -0.768868), (-0.995898, 0.0904832), (-0.956372, 0.29215),
    (0.990545, 0.137189), (0.659118, 0.75204), (-0.0398563, -0.999205), (-0.679682, -0.733507),
    (-0.540035, 0.841643), (-0.0501135, -0.998744), (-0.196305, -0.980543), (0.56964, 0.821894),
    (-0.703653, 0.710544), (0.162676, -0.98668), (-0.919545, 0.392984), (0.805179, 0.593031),
    (0.998757, 0.0498406), (0.358168, 0.933657), (-0.611152, -0.791513), (-0.440479, 0.897763),
    (0.292587, 0.956239), (-0.217415, -0.976079), (-0.252622, -0.967565), (-0.679998, -0.733214),
    (0.402652, -0.915353), (-0.993189, 0.116516), (0.0634956, 0.997982), (0.432309, -0.901725),
    (0.923434, 0.383756), (-0.502058, 0.864834), (0.935584, -0.353105), (0.912111, -0.409944),
    (-0.298643, 0.954365), (-0.796165, 0.60508), (-0.741295, -0.671179), (0.856386, 0.516336),
    (-0.515876, -0.856663), (0.994745, -0.102384), (0.648698, -0.761046), (-0.999675, -0.0254761),
    (-0.130115, 0.991499), (-0.998787, 0.0492413), (0.27449, -0.96159), (-0.996501, -0.0835749),
    (-0.387182, 0.922003), (0.701006, -0.713156), (0.98733, -0.158679), (-0.713847, 0.700302),
    (-0.329606, 0.944118), (0.279362, 0.960186), (-0.968574, -0.248726), (-0.68131, 0.731995),
    (0.220789, -0.975322), (-0.985566, -0.169292), (0.0132834, 0.999912), (-0.422317, 0.906448),
    (-0.772023, -0.635594), (0.842036, -0.539421), (-0.803312, 0.595558), (0.725035, 0.688712),
    (0.328206, 0.944606), (0.711898, 0.702283), (-0.691674, 0.72221), (-0.871274, 0.490797),
    (0.213736, 0.976891), (0.255845, 0.966718), (0.883381, 0.468656), (-0.596736, 0.802437),
    (0.779861, 0.625953), (-0.607233, 0.794524), (-0.944679, -0.327996), (0.851219, -0.52481),
    (-0.859337, -0.51141), (0.953486, -0.301437), (0.512244, -0.85884), (0.160393, 0.987053),
    (0.752002, 0.659161), (0.999882, 0.0153624), (0.778011, -0.628251), (0.9293, -0.369326),
    (-0.605896, 0.795544), (0.633164, 0.774017), (-0.923382, -0.383883), (-0.790911, 0.611931),
    (0.673492, 0.739195), (-0.784902, 0.61962), (0.289472, 0.957187), (0.605387, -0.795931),
    (-0.459844, -0.888), (-0.990035, 0.140822), (-0.686367, -0.727256), (-0.549857, 0.835259),
    (0.90982, 0.415003), (-0.42105, 0.907037), (-0.07295, 0.997336), (-0.24021, 0.970721),
    (0.993154, -0.116813), (-0.59563, -0.803259), (0.526545, -0.850147), (0.9987, -0.0509667),
    (-0.85017, -0.526508), (-0.818838, -0.574025), (0.982094, 0.188394), (0.577634, -0.816296),
    (-0.418394, -0.908265), (0.62868, -0.777664), (-0.118173, -0.992993), (0.896113, 0.443827),
    (-0.159857, -0.98714), (0.750036, 0.661397), (0.745659, 0.666328), (-0.938865, -0.344285),
    (-0.583143, 0.812369), (0.479122, 0.877748), (-0.869898, -0.493231), (-0.791797, 0.610784),
    (0.35785, 0.933779), (0.25248, -0.967602), (-0.573942, -0.818896), (-0.930426, -0.366479),
    (-0.378158, 0.925741), (-0.942114, 0.335293), (0.647836, -0.76178), (0.814174, 0.580622),
    (0.023769, 0.999717), (0.112026, -0.993705), (0.659378, -0.751811), (-0.615064, -0.788477),
    (-0.00328067, 0.999995), (0.902263, -0.431186), (0.201174, -0.979556), (0.541589, 0.840644),
    (-0.996013, 0.0892081), (0.987237, 0.159261), (-0.692353, 0.721559), (0.940855, -0.338809),
    (0.164224, -0.986423), (0.0618662, 0.998084), (0.784694, 0.619883), (0.156281, -0.987713),
    (-0.424548, -0.905405), (0.927622, 0.37352), (-0.923711, 0.383089), (0.708767, -0.705442),
    (-0.941076, 0.338195), (-0.268226, 0.963356), (0.653964, 0.756526), (0.983767, -0.179452),
    (-0.480029, 0.877253), (-0.845565, -0.533873), (-0.768586, -0.639746), (0.208936, 0.977929),
    (0.512539, -0.858664), (0.988163, -0.153409), (0.780816, 0.624761), (-0.232748, 0.972537),
    (0.988528, -0.151035), (-0.106602, -0.994302), (-0.633295, 0.773911), (0.322068, -0.946717),
    (0.989632, -0.143627), (0.973492, -0.228721), (0.998266, 0.0588631), (-0.145619, 0.989341),
];

pub static QWIN: [f64; 640] = [
    0.0, 0.0001990318758627504, 0.0002494762615491542, 0.0003021769445225078, 0.0003548460080857985, 0.0004058915811480806,
    0.0004546408052001889, 0.0005012680176678405, 0.0005464958142195282, 0.0005912073950641334, 0.0006361178026937039, 0.0006816060488244358,
    0.000727725709506429, 0.0007743418255606097, 0.0008212990636826637, 0.0008685363488152327, 0.0009161071539925993, 0.0009641168291303352,
    0.001012630507392736, 0.00106160525810862, 0.001110882587090581, 0.001160236901298543, 0.001209448942573337, 0.001258362795150757,
    0.001306902381715039, 0.001355046337751365, 0.00140278462956841, 0.001450086694843816, 0.001496898951224534, 0.001543170821958483,
    0.001588889089195869, 0.001634098242730728, 0.00167889237249393, 0.00172338117392066, 0.001767651163797991, 0.00181174199861474,
    0.0018556506065872, 0.00189936091508362, 0.001942876625831283, 0.001986241654706626, 0.002029534125962055, 0.002072840712410525,
    0.002116229103721749, 0.002159738034390673, 0.002203392976200947, 0.002247239773881968, 0.002291373966775394, 0.002335946110021889,
    0.002381132815654862, 0.00242708673297629, 0.002473891839822582, 0.002521550367974952, 0.002570013995199655, 0.002619244058999978,
    0.002669265893796866, 0.002720177146231281, 0.00277208884967978, 0.00282500949416298, 0.00287871654406114, 0.002932677076291194,
    0.002986067366389476, 0.003037905983043366, 0.003087269477594307, 0.003133519274378684, 0.003176460810085721, 0.003216374095471449,
    0.003253902493849856, 0.003289837867273167, 0.003324873276103132, 0.003359407689115599, 0.003393454084675361, 0.003426668323773391,
    0.00345846581599975, 0.003488171121469781, 0.00351514135133878, 0.003538827383683883, 0.003558767785536742, 0.003574539247363964,
    0.003585697968628984, 0.003591743339500398, 0.003592116764752254, 0.003586228204993297, 0.003573492966885132, 0.003553356715665694,
    0.003525300399274114, 0.00348882409293152, 0.003443423145747434, 0.003388568319085867, 0.003323699442173841, 0.003248231770523395,
    0.003161568930730635, 0.00306311366696767, 0.002952270973359112, 0.002828441943181057, 0.0026910161732885, 0.002539366102140493,
    0.002372848583221744, 0.002190814088754598, 0.001992618085548526, 0.001777631090142623, 0.001545242163079598, 0.001294855985911958,
    0.001025885587325796, 0.0007377456851538827, 0.0004298496740962311, 0.0001016113723823784, -0.000247549381453534, -0.0006181972580227641,
    -0.001010876063031582, -0.001426108207321696, -0.001864392667409557, -0.002326207721179968, -0.002812013688448634, -0.003322252633537029,
    -0.003857344314546718, -0.004417678415707104, -0.005003604409245843, -0.00561542242754085, -0.006253382198869787, -0.006917691380307223,
    -0.007608536937561301, -0.008326113472848558, -0.009070651572928327, -0.009842433610911637, -0.01064178450184536, -0.01146903570409307,
    -0.01232446526717138, -0.01320822893615923, 0.01412030102138547, 0.01506045143737221, 0.01602824700934038, 0.01702310507234504,
    0.01804435938034114, 0.01909132707403387, 0.02016335321815832, 0.02125982139139435, 0.02238013015948307, 0.02352365148441367,
    0.02468968228813486, 0.02587741357605385, 0.02708591966384863, 0.02831416731612567, 0.02956103453432552, 0.03082532788511644,
    0.03210578787607558, 0.03340108247607704, 0.03470979250147262, 0.03603039785904666, 0.03736126987823528, 0.0387006742898075,
    0.0400467799430386, 0.04139766786359423, 0.04275134353925827, 0.04410572893128047, 0.04545866171224587, 0.04680788921400311,
    0.04815106534667384, 0.04948575188369231, 0.05080942296260306, 0.05211947012173918, 0.05341320372603929, 0.05468785186395163,
    0.05594055607104873, 0.05716836923188953, 0.05836825629443718, 0.0595370994576593, 0.06067170625396996, 0.06176881705202805,
    0.0628251099982746, 0.06383720245755561, 0.06480165083585107, 0.06571495100350305, 0.06657354346196487, 0.06737381445564891,
    0.06811211000439976, 0.06878473991370719, 0.06938797895654626, 0.0699180661858, 0.07037120381110623, 0.07074355866301175,
    0.07103126866531538, 0.0712304556339945, 0.0713372388815184, 0.07134774334517399, 0.07125810128129656, 0.07106444395777428,
    0.07076288963679085, 0.07034953453342756, 0.06982045490146145, 0.06917172452383333, 0.06839944399575645, 0.06749977716975541,
    0.06646898181809889, 0.06530342654389223, 0.06399958984339946, 0.06255404354954748, 0.06096342863203985, 0.05922443337469448,
    0.05733378365410422, 0.05528824660015738, 0.05308464739461209, 0.05071989148277166, 0.04819098634672628, 0.04549505579582869,
    0.04262934676625042, 0.03959122947020497, 0.03637819581239452, 0.03298786054608736, 0.029417969544798, 0.02566640058060906,
    0.02173117939155709, 0.01761048656968719, 0.01330266415707108, 0.008806217289921706, 0.004119815918461287, -0.0007577038291607129,
    -0.005827337082489678, -0.01108990619665782, -0.01654605559674886, -0.02219624707735291, -0.02804075556277473, -0.03407966641908426,
    -0.04031287253355741, -0.04674007190475649, -0.05336076390182971, -0.0601742452694062, -0.06717960594283154, -0.07437572538762392,
    -0.08176127022450692, -0.08933469320120192, -0.0970942330904345, -0.1050379143754414, -0.1131635475471188, -0.1214687284677367,
    -0.1299508386078101, -0.1386070430802319, -0.1474342913196958, -0.1564293167898782, -0.1655886374953163, -0.1749085568711785,
    -0.184385164211629, -0.1940143360850268, -0.2037917371113644, -0.2137128217101543, -0.2237728356363325, -0.2339668182208061,
    -0.2442896055908444, -0.2547358344658102, -0.2652999476893712, -0.275976200367384, -0.2867586659726799, -0.2976412485679301,
    -0.308617682772183, -0.3196815399704708, -0.3308262316588501, -0.3420450091826495, 0.3533309414505971, 0.3646770149404552,
    0.3760759747758828, 0.3875204555118187, 0.3990029533969267, 0.4105158411581483, 0.4220513789540003, 0.433601725130598,
    0.4451589452332786, 0.4567150149423557, 0.4682618290579831, 0.4797912086537587, 0.4912949058677955, 0.5027646134968753,
    0.5141919746376279, 0.5255685924518015, 0.5368860394090674, 0.5481358656081351, 0.5593096071830315, 0.5703987947306394,
    0.5813949615434598, 0.5922896536434017, 0.6030744392774144, 0.6137409201916185, 0.6242807411441345, 0.6346855991963545,
    0.64494725318366, 0.6550575323798634, 0.6650083455855346, 0.6747916901830467, 0.6843996616799759, 0.6938244627003839,
    0.7030584122393319, 0.712093953724119, 0.7209236637533725, 0.729540259902981, 0.7379366091028713, 0.7461057359576386,
    0.754040831494223, 0.761735261150446, 0.769182571458689, 0.7763765020733762, 0.7833109874824341, 0.7899801646390305,
    0.7963783815797485, 0.8025002033685581, 0.8083404191294724, 0.8138940486031526, 0.8191563476989879, 0.8241228138607196,
    0.8287891904413357, 0.8331514714928793, 0.8372059062705359, 0.8409490040631689, 0.8443775395556067, 0.8474885573145614,
    0.8502793750759253, 0.852747586359539, 0.854891060659457, 0.8567079441260879, 0.8581966597760032, 0.8593559096378087,
    0.8601846769933608, 0.8606822313166693, 0.8608481078185763, 0.8606822313166693, 0.8601846769933608, 0.8593559096378087,
    0.8581966597760032, 0.8567079441260879, 0.854891060659457, 0.852747586359539, 0.8502793750759253, 0.8474885573145614,
    0.8443775395556067, 0.8409490040631689, 0.8372059062705359, 0.8331514714928793, 0.8287891904413357, 0.8241228138607196,
    0.8191563476989879, 0.8138940486031526, 0.8083404191294724, 0.8025002033685581, 0.7963783815797485, 0.7899801646390305,
    0.7833109874824341, 0.7763765020733762, 0.769182571458689, 0.761735261150446, 0.754040831494223, 0.7461057359576386,
    0.7379366091028713, 0.729540259902981, 0.7209236637533725, 0.712093953724119, 0.7030584122393319, 0.6938244627003839,
    0.6843996616799759, 0.6747916901830467, 0.6650083455855346, 0.6550575323798634, 0.64494725318366, 0.6346855991963545,
    0.6242807411441345, 0.6137409201916185, 0.6030744392774144, 0.5922896536434017, 0.5813949615434598, 0.5703987947306394,
    0.5593096071830315, 0.5481358656081351, 0.5368860394090674, 0.5255685924518015, 0.5141919746376279, 0.5027646134968753,
    0.4912949058677955, 0.4797912086537587, 0.4682618290579831, 0.4567150149423557, 0.4451589452332786, 0.433601725130598,
    0.4220513789540003, 0.4105158411581483, 0.3990029533969267, 0.3875204555118187, 0.3760759747758828, 0.3646770149404552,
    -0.3533309414505971, -0.3420450091826495, -0.3308262316588501, -0.3196815399704708, -0.308617682772183, -0.2976412485679301,
    -0.2867586659726799, -0.275976200367384, -0.2652999476893712, -0.2547358344658102, -0.2442896055908444, -0.2339668182208061,
    -0.2237728356363325, -0.2137128217101543, -0.2037917371113644, -0.1940143360850268, -0.184385164211629, -0.1749085568711785,
    -0.1655886374953163, -0.1564293167898782, -0.1474342913196958, -0.1386070430802319, -0.1299508386078101, -0.1214687284677367,
    -0.1131635475471188, -0.1050379143754414, -0.0970942330904345, -0.08933469320120192, -0.08176127022450692, -0.07437572538762392,
    -0.06717960594283154, -0.0601742452694062, -0.05336076390182971, -0.04674007190475649, -0.04031287253355741, -0.03407966641908426,
    -0.02804075556277473, -0.02219624707735291, -0.01654605559674886, -0.01108990619665782, -0.005827337082489678, -0.0007577038291607129,
    0.004119815918461287, 0.008806217289921706, 0.01330266415707108, 0.01761048656968719, 0.02173117939155709, 0.02566640058060906,
    0.029417969544798, 0.03298786054608736, 0.03637819581239452, 0.03959122947020497, 0.04262934676625042, 0.04549505579582869,
    0.04819098634672628, 0.05071989148277166, 0.05308464739461209, 0.05528824660015738, 0.05733378365410422, 0.05922443337469448,
    0.06096342863203985, 0.06255404354954748, 0.06399958984339946, 0.06530342654389223, 0.06646898181809889, 0.06749977716975541,
    0.06839944399575645, 0.06917172452383333, 0.06982045490146145, 0.07034953453342756, 0.07076288963679085, 0.07106444395777428,
    0.07125810128129656, 0.07134774334517399, 0.0713372388815184, 0.0712304556339945, 0.07103126866531538, 0.07074355866301175,
    0.07037120381110623, 0.0699180661858, 0.06938797895654626, 0.06878473991370719, 0.06811211000439976, 0.06737381445564891,
    0.06657354346196487, 0.06571495100350305, 0.06480165083585107, 0.06383720245755561, 0.0628251099982746, 0.06176881705202805,
    0.06067170625396996, 0.0595370994576593, 0.05836825629443718, 0.05716836923188953, 0.05594055607104873, 0.05468785186395163,
    0.05341320372603929, 0.05211947012173918, 0.05080942296260306, 0.04948575188369231, 0.04815106534667384, 0.04680788921400311,
    0.04545866171224587, 0.04410572893128047, 0.04275134353925827, 0.04139766786359423, 0.0400467799430386, 0.0387006742898075,
    0.03736126987823528, 0.03603039785904666, 0.03470979250147262, 0.03340108247607704, 0.03210578787607558, 0.03082532788511644,
    0.02956103453432552, 0.02831416731612567, 0.02708591966384863, 0.02587741357605385, 0.02468968228813486, 0.02352365148441367,
    0.02238013015948307, 0.02125982139139435, 0.02016335321815832, 0.01909132707403387, 0.01804435938034114, 0.01702310507234504,
    0.01602824700934038, 0.01506045143737221, -0.01412030102138547, -0.01320822893615923, -0.01232446526717138, -0.01146903570409307,
    -0.01064178450184536, -0.009842433610911637, -0.009070651572928327, -0.008326113472848558, -0.007608536937561301, -0.006917691380307223,
    -0.006253382198869787, -0.00561542242754085, -0.005003604409245843, -0.004417678415707104, -0.003857344314546718, -0.003322252633537029,
    -0.002812013688448634, -0.002326207721179968, -0.001864392667409557, -0.001426108207321696, -0.001010876063031582, -0.0006181972580227641,
    -0.000247549381453534, 0.0001016113723823784, 0.0004298496740962311, 0.0007377456851538827, 0.001025885587325796, 0.001294855985911958,
    0.001545242163079598, 0.001777631090142623, 0.001992618085548526, 0.002190814088754598, 0.002372848583221744, 0.002539366102140493,
    0.0026910161732885, 0.002828441943181057, 0.002952270973359112, 0.00306311366696767, 0.003161568930730635, 0.003248231770523395,
    0.003323699442173841, 0.003388568319085867, 0.003443423145747434, 0.00348882409293152, 0.003525300399274114, 0.003553356715665694,
    0.003573492966885132, 0.003586228204993297, 0.003592116764752254, 0.003591743339500398, 0.003585697968628984, 0.003574539247363964,
    0.003558767785536742, 0.003538827383683883, 0.00351514135133878, 0.003488171121469781, 0.00345846581599975, 0.003426668323773391,
    0.003393454084675361, 0.003359407689115599, 0.003324873276103132, 0.003289837867273167, 0.003253902493849856, 0.003216374095471449,
    0.003176460810085721, 0.003133519274378684, 0.003087269477594307, 0.003037905983043366, 0.002986067366389476, 0.002932677076291194,
    0.00287871654406114, 0.00282500949416298, 0.00277208884967978, 0.002720177146231281, 0.002669265893796866, 0.002619244058999978,
    0.002570013995199655, 0.002521550367974952, 0.002473891839822582, 0.00242708673297629, 0.002381132815654862, 0.002335946110021889,
    0.002291373966775394, 0.002247239773881968, 0.002203392976200947, 0.002159738034390673, 0.002116229103721749, 0.002072840712410525,
    0.002029534125962055, 0.001986241654706626, 0.001942876625831283, 0.00189936091508362, 0.0018556506065872, 0.00181174199861474,
    0.001767651163797991, 0.00172338117392066, 0.00167889237249393, 0.001634098242730728, 0.001588889089195869, 0.001543170821958483,
    0.001496898951224534, 0.001450086694843816, 0.00140278462956841, 0.001355046337751365, 0.001306902381715039, 0.001258362795150757,
    0.001209448942573337, 0.001160236901298543, 0.001110882587090581, 0.00106160525810862, 0.001012630507392736, 0.0009641168291303352,
    0.0009161071539925993, 0.0008685363488152327, 0.0008212990636826637, 0.0007743418255606097, 0.000727725709506429, 0.0006816060488244358,
    0.0006361178026937039, 0.0005912073950641334, 0.0005464958142195282, 0.0005012680176678405, 0.0004546408052001889, 0.0004058915811480806,
    0.0003548460080857985, 0.0003021769445225078, 0.0002494762615491542, 0.0001990318758627504,
];
