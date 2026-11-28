//! AC-4 Huffman 码表实例缓存（按需懒加载）。

use std::sync::OnceLock;

use crate::bits::Huffman;
use crate::tables;

macro_rules! define_books {
    ($($variant:ident => $name:literal),* $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Book {
            $($variant,)*
        }

        pub fn huffman(book: Book) -> &'static Huffman {
            match book {
                $(
                    Book::$variant => {
                        static CELL: OnceLock<Huffman> = OnceLock::new();
                        CELL.get_or_init(|| {
                            let cb = tables::codebook($name).expect("missing AC-4 codebook");
                            Huffman::from_codebook(cb)
                        })
                    }
                )*
            }
        }
    };
}

define_books! {
    Asf1 => "ASF_HCB_1",
    Asf2 => "ASF_HCB_2",
    Asf3 => "ASF_HCB_3",
    Asf4 => "ASF_HCB_4",
    Asf5 => "ASF_HCB_5",
    Asf6 => "ASF_HCB_6",
    Asf7 => "ASF_HCB_7",
    Asf8 => "ASF_HCB_8",
    Asf9 => "ASF_HCB_9",
    Asf10 => "ASF_HCB_10",
    Asf11 => "ASF_HCB_11",
    AsfScalefac => "ASF_HCB_SCALEFAC",
    AsfSnf => "ASF_HCB_SNF",

    AspxEnvLevel15F0 => "ASPX_HCB_ENV_LEVEL_15_F0",
    AspxEnvLevel15Df => "ASPX_HCB_ENV_LEVEL_15_DF",
    AspxEnvLevel15Dt => "ASPX_HCB_ENV_LEVEL_15_DT",
    AspxEnvLevel30F0 => "ASPX_HCB_ENV_LEVEL_30_F0",
    AspxEnvLevel30Df => "ASPX_HCB_ENV_LEVEL_30_DF",
    AspxEnvLevel30Dt => "ASPX_HCB_ENV_LEVEL_30_DT",
    AspxEnvBalance15F0 => "ASPX_HCB_ENV_BALANCE_15_F0",
    AspxEnvBalance15Df => "ASPX_HCB_ENV_BALANCE_15_DF",
    AspxEnvBalance15Dt => "ASPX_HCB_ENV_BALANCE_15_DT",
    AspxEnvBalance30F0 => "ASPX_HCB_ENV_BALANCE_30_F0",
    AspxEnvBalance30Df => "ASPX_HCB_ENV_BALANCE_30_DF",
    AspxEnvBalance30Dt => "ASPX_HCB_ENV_BALANCE_30_DT",
    AspxNoiseLevelF0 => "ASPX_HCB_NOISE_LEVEL_F0",
    AspxNoiseLevelDf => "ASPX_HCB_NOISE_LEVEL_DF",
    AspxNoiseLevelDt => "ASPX_HCB_NOISE_LEVEL_DT",
    AspxNoiseBalanceF0 => "ASPX_HCB_NOISE_BALANCE_F0",
    AspxNoiseBalanceDf => "ASPX_HCB_NOISE_BALANCE_DF",
    AspxNoiseBalanceDt => "ASPX_HCB_NOISE_BALANCE_DT",

    AcplAlphaCoarseF0 => "ACPL_HCB_ALPHA_COARSE_F0",
    AcplAlphaCoarseDf => "ACPL_HCB_ALPHA_COARSE_DF",
    AcplAlphaCoarseDt => "ACPL_HCB_ALPHA_COARSE_DT",
    AcplAlphaFineF0 => "ACPL_HCB_ALPHA_FINE_F0",
    AcplAlphaFineDf => "ACPL_HCB_ALPHA_FINE_DF",
    AcplAlphaFineDt => "ACPL_HCB_ALPHA_FINE_DT",
    AcplBetaCoarseF0 => "ACPL_HCB_BETA_COARSE_F0",
    AcplBetaCoarseDf => "ACPL_HCB_BETA_COARSE_DF",
    AcplBetaCoarseDt => "ACPL_HCB_BETA_COARSE_DT",
    AcplBetaFineF0 => "ACPL_HCB_BETA_FINE_F0",
    AcplBetaFineDf => "ACPL_HCB_BETA_FINE_DF",
    AcplBetaFineDt => "ACPL_HCB_BETA_FINE_DT",
    AcplBeta3CoarseF0 => "ACPL_HCB_BETA3_COARSE_F0",
    AcplBeta3CoarseDf => "ACPL_HCB_BETA3_COARSE_DF",
    AcplBeta3CoarseDt => "ACPL_HCB_BETA3_COARSE_DT",
    AcplBeta3FineF0 => "ACPL_HCB_BETA3_FINE_F0",
    AcplBeta3FineDf => "ACPL_HCB_BETA3_FINE_DF",
    AcplBeta3FineDt => "ACPL_HCB_BETA3_FINE_DT",
    AcplGammaCoarseF0 => "ACPL_HCB_GAMMA_COARSE_F0",
    AcplGammaCoarseDf => "ACPL_HCB_GAMMA_COARSE_DF",
    AcplGammaCoarseDt => "ACPL_HCB_GAMMA_COARSE_DT",
    AcplGammaFineF0 => "ACPL_HCB_GAMMA_FINE_F0",
    AcplGammaFineDf => "ACPL_HCB_GAMMA_FINE_DF",
    AcplGammaFineDt => "ACPL_HCB_GAMMA_FINE_DT",

    AjocDryCoarseF0 => "AJOC_HCB_DRY_COARSE_F0",
    AjocDryCoarseDf => "AJOC_HCB_DRY_COARSE_DF",
    AjocDryCoarseDt => "AJOC_HCB_DRY_COARSE_DT",
    AjocDryFineF0 => "AJOC_HCB_DRY_FINE_F0",
    AjocDryFineDf => "AJOC_HCB_DRY_FINE_DF",
    AjocDryFineDt => "AJOC_HCB_DRY_FINE_DT",
    AjocWetCoarseF0 => "AJOC_HCB_WET_COARSE_F0",
    AjocWetCoarseDf => "AJOC_HCB_WET_COARSE_DF",
    AjocWetCoarseDt => "AJOC_HCB_WET_COARSE_DT",
    AjocWetFineF0 => "AJOC_HCB_WET_FINE_F0",
    AjocWetFineDf => "AJOC_HCB_WET_FINE_DF",
    AjocWetFineDt => "AJOC_HCB_WET_FINE_DT",
}

pub fn asf_book(cb: usize) -> Book {
    match cb {
        1 => Book::Asf1,
        2 => Book::Asf2,
        3 => Book::Asf3,
        4 => Book::Asf4,
        5 => Book::Asf5,
        6 => Book::Asf6,
        7 => Book::Asf7,
        8 => Book::Asf8,
        9 => Book::Asf9,
        10 => Book::Asf10,
        11 => Book::Asf11,
        _ => panic!("invalid ASF codebook {cb}"),
    }
}
