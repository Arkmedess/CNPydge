//! Conversor colunar unificado de alta vazão para Apache Parquet.

use crate::dominio::Dominio;
use crate::empresa::Empresa;
use crate::estabelecimento::Estabelecimento;
use crate::reader::MmapReader;
use crate::simples::Simples;
use crate::socio::Socio;
use arrow::array::{
    ArrayRef, Float64Builder, StringBuilder, UInt8Builder, UInt16Builder, UInt32Builder,
};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;
use rayon::prelude::*;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

/// Erros tipados durante a leitura, conversão colunar e gravação do Parquet.
#[derive(Debug, thiserror::Error)]
pub enum ConvertErr {
    /// Falha de I/O na leitura do arquivo de origem ou criação do arquivo de destino.
    #[error("Erro de I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Falha durante a construção de arrays ou batches no Apache Arrow.
    #[error("Erro no Arrow: {0}")]
    Arrow(#[from] arrow::error::ArrowError),
    /// Falha na serialização ou escrita do arquivo Apache Parquet.
    #[error("Erro no Parquet: {0}")]
    Parquet(#[from] parquet::errors::ParquetError),
    /// Identificador de entidade não suportado pelo conversor.
    #[error("Tabela não suportada: {0}")]
    UnsupportedTable(String),
}

/// Identificador fortemente tipado das tabelas do CNPJ suportadas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableKind {
    /// Tabela principal de dados cadastrais de Empresas.
    Empresas,
    /// Tabela do Quadro de Sócios e Administradores (QSA).
    Socios,
    /// Tabela de unidades e dados de Estabelecimentos (matriz/filial).
    Estabelecimentos,
    /// Tabela de histórico e opção pelo Simples Nacional e MEI.
    Simples,
    /// Tabela de Classificação Nacional de Atividades Econômicas.
    Cnaes,
    /// Tabela de motivos de situação cadastral.
    Motivos,
    /// Tabela de municípios da Receita Federal / SIAFI.
    Municipios,
    /// Tabela de naturezas jurídicas.
    Naturezas,
    /// Tabela de países segundo a Receita Federal.
    Paises,
    /// Tabela de qualificações de sócios e representantes legais.
    Qualificacoes,
}

impl TableKind {
    /// Converte string para a variante de tabela correspondente.
    ///
    /// ### Parâmetros
    /// - `s`: Identificador textual da tabela (case-insensitive, ex.: `"empresas"`, `"socios"`).
    ///
    /// ### Retorno
    /// Variante `TableKind` mapeada.
    ///
    /// ### Erros
    /// Retorna `ConvertErr::UnsupportedTable` caso o identificador não seja reconhecido.
    pub fn parse(s: &str) -> Result<Self, ConvertErr> {
        match s.trim().to_lowercase().as_str() {
            "empresas" | "empresa" => Ok(Self::Empresas),
            "socios" | "socio" => Ok(Self::Socios),
            "estabelecimentos" | "estabelecimento" => Ok(Self::Estabelecimentos),
            "simples" | "simples_nacional" | "mei" => Ok(Self::Simples),
            "cnaes" | "cnae" => Ok(Self::Cnaes),
            "motivos" | "motivo" => Ok(Self::Motivos),
            "municipios" | "municipio" => Ok(Self::Municipios),
            "naturezas" | "natureza" => Ok(Self::Naturezas),
            "paises" | "pais" => Ok(Self::Paises),
            "qualificacoes" | "qualificacao" => Ok(Self::Qualificacoes),
            _ => Err(ConvertErr::UnsupportedTable(s.to_string())),
        }
    }

    /// Retorna o esquema colunar Apache Arrow específico da tabela.
    ///
    /// ### Retorno
    /// Referência compartilhada imutável (`SchemaRef`) para o esquema Arrow configurado.
    pub fn schema(&self) -> SchemaRef {
        match self {
            Self::Empresas => Arc::new(Schema::new(vec![
                Field::new("cnpj_basico", DataType::Utf8, false),
                Field::new("razao_social", DataType::Utf8, false),
                Field::new("natureza_juridica", DataType::UInt16, false),
                Field::new("qualificacao_responsavel", DataType::UInt16, false),
                Field::new("capital_social", DataType::Float64, false),
                Field::new("porte", DataType::UInt8, false),
                Field::new("ente_federativo", DataType::Utf8, true),
            ])),
            Self::Socios => Arc::new(Schema::new(vec![
                Field::new("cnpj_basico", DataType::Utf8, false),
                Field::new("identificador_socio", DataType::UInt8, false),
                Field::new("nome_socio", DataType::Utf8, false),
                Field::new("cpf_cnpj_socio", DataType::Utf8, false),
                Field::new("qualificacao_socio", DataType::UInt16, false),
                Field::new("data_entrada_sociedade", DataType::UInt32, false),
                Field::new("pais", DataType::UInt16, true),
                Field::new("representante_legal", DataType::Utf8, true),
                Field::new("nome_representante", DataType::Utf8, true),
                Field::new("qualificacao_representante", DataType::UInt16, true),
                Field::new("faixa_etaria", DataType::UInt8, false),
            ])),
            Self::Estabelecimentos => Arc::new(Schema::new(vec![
                Field::new("cnpj_basico", DataType::Utf8, false),
                Field::new("cnpj_ordem", DataType::Utf8, false),
                Field::new("cnpj_dv", DataType::Utf8, false),
                Field::new("matriz_filial", DataType::UInt8, false),
                Field::new("nome_fantasia", DataType::Utf8, true),
                Field::new("situacao_cadastral", DataType::UInt8, false),
                Field::new("data_situacao_cadastral", DataType::UInt32, true),
                Field::new("motivo_situacao_cadastral", DataType::UInt16, false),
                Field::new("nome_cidade_exterior", DataType::Utf8, true),
                Field::new("pais", DataType::UInt16, true),
                Field::new("data_inicio_atividade", DataType::UInt32, true),
                Field::new("cnae_fiscal_principal", DataType::UInt32, false),
                Field::new("cnae_fiscal_secundaria", DataType::Utf8, true),
                Field::new("tipo_logradouro", DataType::Utf8, true),
                Field::new("logradouro", DataType::Utf8, true),
                Field::new("numero", DataType::Utf8, true),
                Field::new("complemento", DataType::Utf8, true),
                Field::new("bairro", DataType::Utf8, true),
                Field::new("cep", DataType::Utf8, true),
                Field::new("uf", DataType::Utf8, true),
                Field::new("municipio", DataType::UInt16, true),
                Field::new("ddd_1", DataType::Utf8, true),
                Field::new("telefone_1", DataType::Utf8, true),
                Field::new("ddd_2", DataType::Utf8, true),
                Field::new("telefone_2", DataType::Utf8, true),
                Field::new("ddd_fax", DataType::Utf8, true),
                Field::new("fax", DataType::Utf8, true),
                Field::new("correio_eletronico", DataType::Utf8, true),
                Field::new("situacao_especial", DataType::Utf8, true),
                Field::new("data_situacao_especial", DataType::UInt32, true),
            ])),
            Self::Simples => Arc::new(Schema::new(vec![
                Field::new("cnpj_basico", DataType::Utf8, false),
                Field::new("opcao_simples", DataType::Utf8, true),
                Field::new("data_opcao_simples", DataType::UInt32, true),
                Field::new("data_exclusao_simples", DataType::UInt32, true),
                Field::new("opcao_mei", DataType::Utf8, true),
                Field::new("data_opcao_mei", DataType::UInt32, true),
                Field::new("data_exclusao_mei", DataType::UInt32, true),
            ])),
            Self::Cnaes => Arc::new(Schema::new(vec![
                Field::new("codigo", DataType::UInt32, false),
                Field::new("descricao", DataType::Utf8, false),
            ])),
            Self::Motivos
            | Self::Municipios
            | Self::Naturezas
            | Self::Paises
            | Self::Qualificacoes => Arc::new(Schema::new(vec![
                Field::new("codigo", DataType::UInt16, false),
                Field::new("descricao", DataType::Utf8, false),
            ])),
        }
    }

    /// Constrói um `RecordBatch` para a fatia de bytes de acordo com o tipo da tabela.
    ///
    /// ### Parâmetros
    /// - `slice`: Fatia de bytes mapeada contendo linhas íntegras do CSV.
    ///
    /// ### Retorno
    /// `Ok(Some(RecordBatch))` com as colunas Arrow povoadas ou `Ok(None)` se a fatia estiver vazia.
    ///
    /// ### Erros
    /// Retorna `ConvertErr` caso haja inconsistência de tipos ou falha de construção do batch.
    pub fn build_batch(&self, slice: &[u8]) -> Result<Option<RecordBatch>, ConvertErr> {
        match self {
            Self::Empresas => build_empresas(slice, &self.schema()),
            Self::Socios => build_socios(slice, &self.schema()),
            Self::Estabelecimentos => build_estabelecimentos(slice, &self.schema()),
            Self::Simples => build_simples(slice, &self.schema()),
            Self::Cnaes => build_dominio_u32(slice, &self.schema()),
            Self::Motivos
            | Self::Municipios
            | Self::Naturezas
            | Self::Paises
            | Self::Qualificacoes => build_dominio_u16(slice, &self.schema()),
        }
    }
}

/// Pipeline unificado de conversão de CSV para Parquet em alta vazão.
///
/// ### Parâmetros
/// - `src`: Caminho de origem do arquivo CSV no disco.
/// - `dst`: Caminho de destino do arquivo Parquet colunar gerado.
/// - `kind`: Variante fortemente tipada `TableKind` definindo o parser e esquema.
///
/// ### Retorno
/// Contagem total de linhas válidas convertidas e gravadas com sucesso.
///
/// ### Erros
/// Retorna `ConvertErr` em caso de erro de leitura mapeada, parsing de fatias ou escrita Parquet.
pub fn to_parquet<P1: AsRef<Path>, P2: AsRef<Path>>(
    src: P1,
    dst: P2,
    kind: TableKind,
) -> Result<usize, ConvertErr> {
    let reader: MmapReader = MmapReader::from_path(src)?;
    if reader.is_empty() {
        return Ok(0);
    }

    let num_threads: usize = rayon::current_num_threads().max(1);
    let boundaries: Vec<(usize, usize)> = reader.chunk_boundaries(num_threads);
    let bytes: &[u8] = reader.as_bytes();

    let batches: Vec<RecordBatch> = boundaries
        .par_iter()
        .filter_map(|&(start, end)| {
            let chunk_slice: &[u8] = &bytes[start..end];
            kind.build_batch(chunk_slice).ok().flatten()
        })
        .collect();

    let total_rows: usize = batches.iter().map(|b: &RecordBatch| b.num_rows()).sum();
    if total_rows == 0 {
        return Ok(0);
    }

    let file: File = File::create(dst)?;
    let props: WriterProperties = WriterProperties::builder()
        .set_compression(Compression::SNAPPY)
        .build();

    let mut writer: ArrowWriter<File> = ArrowWriter::try_new(file, kind.schema(), Some(props))?;
    for batch in &batches {
        writer.write(batch)?;
    }
    writer.close()?;

    Ok(total_rows)
}

fn build_empresas(slice: &[u8], schema: &SchemaRef) -> Result<Option<RecordBatch>, ConvertErr> {
    if slice.is_empty() {
        return Ok(None);
    }
    let mut cnpj_b: StringBuilder = StringBuilder::new();
    let mut razao_b: StringBuilder = StringBuilder::new();
    let mut nat_jur_b: UInt16Builder = UInt16Builder::new();
    let mut qualif_b: UInt16Builder = UInt16Builder::new();
    let mut capital_b: Float64Builder = Float64Builder::new();
    let mut porte_b: UInt8Builder = UInt8Builder::new();
    let mut ente_fed_b: StringBuilder = StringBuilder::new();
    let mut count: usize = 0;
    let mut start: usize = 0;

    for (i, &byte) in slice.iter().enumerate() {
        if byte == b'\n' {
            let line: &[u8] = &slice[start..i];
            start = i + 1;
            if line.is_empty() || (line.len() == 1 && line[0] == b'\r') {
                continue;
            }
            let Ok(empresa) = Empresa::parse_line(line) else {
                continue;
            };
            let Ok(cnpj_str) = std::str::from_utf8(empresa.cnpj) else {
                continue;
            };

            cnpj_b.append_value(cnpj_str);
            razao_b.append_value(empresa.razao);
            nat_jur_b.append_value(empresa.nat_jur);
            qualif_b.append_value(empresa.qualif);
            capital_b.append_value(empresa.capital);
            porte_b.append_value(empresa.porte);
            match empresa.ente_fed {
                Some(ente) => ente_fed_b.append_value(ente),
                None => ente_fed_b.append_null(),
            }
            count += 1;
        }
    }
    if count == 0 {
        return Ok(None);
    }
    let columns: Vec<ArrayRef> = vec![
        Arc::new(cnpj_b.finish()),
        Arc::new(razao_b.finish()),
        Arc::new(nat_jur_b.finish()),
        Arc::new(qualif_b.finish()),
        Arc::new(capital_b.finish()),
        Arc::new(porte_b.finish()),
        Arc::new(ente_fed_b.finish()),
    ];
    Ok(Some(RecordBatch::try_new(Arc::clone(schema), columns)?))
}

fn build_socios(slice: &[u8], schema: &SchemaRef) -> Result<Option<RecordBatch>, ConvertErr> {
    if slice.is_empty() {
        return Ok(None);
    }
    let mut cnpj_b: StringBuilder = StringBuilder::new();
    let mut tipo_socio_b: UInt8Builder = UInt8Builder::new();
    let mut nome_b: StringBuilder = StringBuilder::new();
    let mut doc_b: StringBuilder = StringBuilder::new();
    let mut qualif_b: UInt16Builder = UInt16Builder::new();
    let mut data_b: UInt32Builder = UInt32Builder::new();
    let mut pais_b: UInt16Builder = UInt16Builder::new();
    let mut rep_legal_b: StringBuilder = StringBuilder::new();
    let mut nome_rep_b: StringBuilder = StringBuilder::new();
    let mut qualif_rep_b: UInt16Builder = UInt16Builder::new();
    let mut faixa_b: UInt8Builder = UInt8Builder::new();
    let mut count: usize = 0;
    let mut start: usize = 0;

    for (i, &byte) in slice.iter().enumerate() {
        if byte == b'\n' {
            let line: &[u8] = &slice[start..i];
            start = i + 1;
            if line.is_empty() || (line.len() == 1 && line[0] == b'\r') {
                continue;
            }
            let Ok(socio) = Socio::parse_line(line) else {
                continue;
            };
            let Ok(cnpj_str) = std::str::from_utf8(socio.cnpj) else {
                continue;
            };

            cnpj_b.append_value(cnpj_str);
            tipo_socio_b.append_value(socio.tipo_socio);
            nome_b.append_value(socio.nome);
            doc_b.append_value(socio.doc_socio);
            qualif_b.append_value(socio.qualif);
            data_b.append_value(socio.data_entrada);
            match socio.pais {
                Some(p) => pais_b.append_value(p),
                None => pais_b.append_null(),
            }
            match socio.rep_legal {
                Some(r) => rep_legal_b.append_value(r),
                None => rep_legal_b.append_null(),
            }
            match socio.nome_rep {
                Some(n) => nome_rep_b.append_value(n),
                None => nome_rep_b.append_null(),
            }
            match socio.qualif_rep {
                Some(q) => qualif_rep_b.append_value(q),
                None => qualif_rep_b.append_null(),
            }
            faixa_b.append_value(socio.faixa_etaria);
            count += 1;
        }
    }
    if count == 0 {
        return Ok(None);
    }
    let columns: Vec<ArrayRef> = vec![
        Arc::new(cnpj_b.finish()),
        Arc::new(tipo_socio_b.finish()),
        Arc::new(nome_b.finish()),
        Arc::new(doc_b.finish()),
        Arc::new(qualif_b.finish()),
        Arc::new(data_b.finish()),
        Arc::new(pais_b.finish()),
        Arc::new(rep_legal_b.finish()),
        Arc::new(nome_rep_b.finish()),
        Arc::new(qualif_rep_b.finish()),
        Arc::new(faixa_b.finish()),
    ];
    Ok(Some(RecordBatch::try_new(Arc::clone(schema), columns)?))
}

fn build_estabelecimentos(
    slice: &[u8],
    schema: &SchemaRef,
) -> Result<Option<RecordBatch>, ConvertErr> {
    if slice.is_empty() {
        return Ok(None);
    }
    let mut cnpj_basico_b: StringBuilder = StringBuilder::new();
    let mut cnpj_ordem_b: StringBuilder = StringBuilder::new();
    let mut cnpj_dv_b: StringBuilder = StringBuilder::new();
    let mut matriz_filial_b: UInt8Builder = UInt8Builder::new();
    let mut fantasia_b: StringBuilder = StringBuilder::new();
    let mut situacao_b: UInt8Builder = UInt8Builder::new();
    let mut data_situacao_b: UInt32Builder = UInt32Builder::new();
    let mut motivo_b: UInt16Builder = UInt16Builder::new();
    let mut cidade_ext_b: StringBuilder = StringBuilder::new();
    let mut pais_b: UInt16Builder = UInt16Builder::new();
    let mut data_inicio_b: UInt32Builder = UInt32Builder::new();
    let mut cnae_princ_b: UInt32Builder = UInt32Builder::new();
    let mut cnae_sec_b: StringBuilder = StringBuilder::new();
    let mut tipo_logr_b: StringBuilder = StringBuilder::new();
    let mut logradouro_b: StringBuilder = StringBuilder::new();
    let mut numero_b: StringBuilder = StringBuilder::new();
    let mut complemento_b: StringBuilder = StringBuilder::new();
    let mut bairro_b: StringBuilder = StringBuilder::new();
    let mut cep_b: StringBuilder = StringBuilder::new();
    let mut uf_b: StringBuilder = StringBuilder::new();
    let mut municipio_b: UInt16Builder = UInt16Builder::new();
    let mut ddd_1_b: StringBuilder = StringBuilder::new();
    let mut tel_1_b: StringBuilder = StringBuilder::new();
    let mut ddd_2_b: StringBuilder = StringBuilder::new();
    let mut tel_2_b: StringBuilder = StringBuilder::new();
    let mut ddd_fax_b: StringBuilder = StringBuilder::new();
    let mut fax_b: StringBuilder = StringBuilder::new();
    let mut email_b: StringBuilder = StringBuilder::new();
    let mut sit_esp_b: StringBuilder = StringBuilder::new();
    let mut data_sit_esp_b: UInt32Builder = UInt32Builder::new();

    let mut count: usize = 0;
    let mut start: usize = 0;

    for (i, &byte) in slice.iter().enumerate() {
        if byte == b'\n' {
            let line: &[u8] = &slice[start..i];
            start = i + 1;
            if line.is_empty() || (line.len() == 1 && line[0] == b'\r') {
                continue;
            }
            let Ok(est) = Estabelecimento::parse_line(line) else {
                continue;
            };
            let Ok(cnpj_str) = std::str::from_utf8(est.cnpj_basico) else {
                continue;
            };
            let Ok(ordem_str) = std::str::from_utf8(est.cnpj_ordem) else {
                continue;
            };
            let Ok(dv_str) = std::str::from_utf8(est.cnpj_dv) else {
                continue;
            };

            cnpj_basico_b.append_value(cnpj_str);
            cnpj_ordem_b.append_value(ordem_str);
            cnpj_dv_b.append_value(dv_str);
            matriz_filial_b.append_value(est.matriz_filial);

            append_opt_str(&mut fantasia_b, est.fantasia);
            situacao_b.append_value(est.situacao);
            append_opt_u32(&mut data_situacao_b, est.data_situacao);
            motivo_b.append_value(est.motivo_situacao);
            append_opt_str(&mut cidade_ext_b, est.cidade_exterior);
            append_opt_u16(&mut pais_b, est.pais);
            append_opt_u32(&mut data_inicio_b, est.data_inicio);
            cnae_princ_b.append_value(est.cnae_principal);
            append_opt_str(&mut cnae_sec_b, est.cnae_secundario);
            append_opt_str(&mut tipo_logr_b, est.tipo_logradouro);
            append_opt_str(&mut logradouro_b, est.logradouro);
            append_opt_str(&mut numero_b, est.numero);
            append_opt_str(&mut complemento_b, est.complemento);
            append_opt_str(&mut bairro_b, est.bairro);
            append_opt_str(&mut cep_b, est.cep);
            append_opt_str(&mut uf_b, est.uf);
            append_opt_u16(&mut municipio_b, est.municipio);
            append_opt_str(&mut ddd_1_b, est.ddd_1);
            append_opt_str(&mut tel_1_b, est.telefone_1);
            append_opt_str(&mut ddd_2_b, est.ddd_2);
            append_opt_str(&mut tel_2_b, est.telefone_2);
            append_opt_str(&mut ddd_fax_b, est.ddd_fax);
            append_opt_str(&mut fax_b, est.fax);
            append_opt_str(&mut email_b, est.email);
            append_opt_str(&mut sit_esp_b, est.situacao_especial);
            append_opt_u32(&mut data_sit_esp_b, est.data_situacao_especial);

            count += 1;
        }
    }
    if count == 0 {
        return Ok(None);
    }
    let columns: Vec<ArrayRef> = vec![
        Arc::new(cnpj_basico_b.finish()),
        Arc::new(cnpj_ordem_b.finish()),
        Arc::new(cnpj_dv_b.finish()),
        Arc::new(matriz_filial_b.finish()),
        Arc::new(fantasia_b.finish()),
        Arc::new(situacao_b.finish()),
        Arc::new(data_situacao_b.finish()),
        Arc::new(motivo_b.finish()),
        Arc::new(cidade_ext_b.finish()),
        Arc::new(pais_b.finish()),
        Arc::new(data_inicio_b.finish()),
        Arc::new(cnae_princ_b.finish()),
        Arc::new(cnae_sec_b.finish()),
        Arc::new(tipo_logr_b.finish()),
        Arc::new(logradouro_b.finish()),
        Arc::new(numero_b.finish()),
        Arc::new(complemento_b.finish()),
        Arc::new(bairro_b.finish()),
        Arc::new(cep_b.finish()),
        Arc::new(uf_b.finish()),
        Arc::new(municipio_b.finish()),
        Arc::new(ddd_1_b.finish()),
        Arc::new(tel_1_b.finish()),
        Arc::new(ddd_2_b.finish()),
        Arc::new(tel_2_b.finish()),
        Arc::new(ddd_fax_b.finish()),
        Arc::new(fax_b.finish()),
        Arc::new(email_b.finish()),
        Arc::new(sit_esp_b.finish()),
        Arc::new(data_sit_esp_b.finish()),
    ];
    Ok(Some(RecordBatch::try_new(Arc::clone(schema), columns)?))
}

fn build_simples(slice: &[u8], schema: &SchemaRef) -> Result<Option<RecordBatch>, ConvertErr> {
    let mut cnpj_b = StringBuilder::new();
    let mut opc_simples_b = StringBuilder::new();
    let mut dt_opc_simples_b = UInt32Builder::new();
    let mut dt_exc_simples_b = UInt32Builder::new();
    let mut opc_mei_b = StringBuilder::new();
    let mut dt_opc_mei_b = UInt32Builder::new();
    let mut dt_exc_mei_b = UInt32Builder::new();

    let mut row_count = 0;
    for line in slice.split(|&b| b == b'\n') {
        let line = if line.ends_with(b"\r") {
            &line[..line.len() - 1]
        } else {
            line
        };
        if line.is_empty() {
            continue;
        }

        let Ok(simples) = Simples::parse_line(line) else {
            continue;
        };
        let Ok(cnpj_str) = std::str::from_utf8(simples.cnpj_basico) else {
            continue;
        };

        cnpj_b.append_value(cnpj_str);
        append_opt_str(&mut opc_simples_b, simples.opcao_simples);
        append_opt_u32(&mut dt_opc_simples_b, simples.data_opcao_simples);
        append_opt_u32(&mut dt_exc_simples_b, simples.data_exclusao_simples);
        append_opt_str(&mut opc_mei_b, simples.opcao_mei);
        append_opt_u32(&mut dt_opc_mei_b, simples.data_opcao_mei);
        append_opt_u32(&mut dt_exc_mei_b, simples.data_exclusao_mei);

        row_count += 1;
    }

    if row_count == 0 {
        return Ok(None);
    }

    let columns: Vec<ArrayRef> = vec![
        Arc::new(cnpj_b.finish()),
        Arc::new(opc_simples_b.finish()),
        Arc::new(dt_opc_simples_b.finish()),
        Arc::new(dt_exc_simples_b.finish()),
        Arc::new(opc_mei_b.finish()),
        Arc::new(dt_opc_mei_b.finish()),
        Arc::new(dt_exc_mei_b.finish()),
    ];
    Ok(Some(RecordBatch::try_new(Arc::clone(schema), columns)?))
}

fn build_dominio_u32(slice: &[u8], schema: &SchemaRef) -> Result<Option<RecordBatch>, ConvertErr> {
    let mut cod_b = UInt32Builder::new();
    let mut desc_b = StringBuilder::new();

    let mut row_count = 0;
    for line in slice.split(|&b| b == b'\n') {
        let line = if line.ends_with(b"\r") {
            &line[..line.len() - 1]
        } else {
            line
        };
        if line.is_empty() {
            continue;
        }

        let Ok(dom) = Dominio::<u32>::parse_line(line) else {
            continue;
        };
        cod_b.append_value(dom.codigo);
        desc_b.append_value(dom.descricao);
        row_count += 1;
    }

    if row_count == 0 {
        return Ok(None);
    }

    let columns: Vec<ArrayRef> = vec![Arc::new(cod_b.finish()), Arc::new(desc_b.finish())];
    Ok(Some(RecordBatch::try_new(Arc::clone(schema), columns)?))
}

fn build_dominio_u16(slice: &[u8], schema: &SchemaRef) -> Result<Option<RecordBatch>, ConvertErr> {
    let mut cod_b = UInt16Builder::new();
    let mut desc_b = StringBuilder::new();

    let mut row_count = 0;
    for line in slice.split(|&b| b == b'\n') {
        let line = if line.ends_with(b"\r") {
            &line[..line.len() - 1]
        } else {
            line
        };
        if line.is_empty() {
            continue;
        }

        let Ok(dom) = Dominio::<u16>::parse_line(line) else {
            continue;
        };
        cod_b.append_value(dom.codigo);
        desc_b.append_value(dom.descricao);
        row_count += 1;
    }

    if row_count == 0 {
        return Ok(None);
    }

    let columns: Vec<ArrayRef> = vec![Arc::new(cod_b.finish()), Arc::new(desc_b.finish())];
    Ok(Some(RecordBatch::try_new(Arc::clone(schema), columns)?))
}

#[inline]
fn append_opt_str(b: &mut StringBuilder, val: Option<&str>) {
    match val {
        Some(v) => b.append_value(v),
        None => b.append_null(),
    }
}

#[inline]
fn append_opt_u16(b: &mut UInt16Builder, val: Option<u16>) {
    match val {
        Some(v) => b.append_value(v),
        None => b.append_null(),
    }
}

#[inline]
fn append_opt_u32(b: &mut UInt32Builder, val: Option<u32>) {
    match val {
        Some(v) => b.append_value(v),
        None => b.append_null(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_table_kind_parse() {
        assert_eq!(TableKind::parse("empresas").unwrap(), TableKind::Empresas);
        assert_eq!(TableKind::parse("SOCIOS").unwrap(), TableKind::Socios);
        assert_eq!(
            TableKind::parse("estabelecimento").unwrap(),
            TableKind::Estabelecimentos
        );
        assert_eq!(TableKind::parse("simples").unwrap(), TableKind::Simples);
        assert!(TableKind::parse("invalida").is_err());
    }

    #[test]
    fn test_unified_to_parquet_empresas() {
        let temp_dir: std::path::PathBuf = std::env::temp_dir();
        let csv: std::path::PathBuf = temp_dir.join("test_unified_empresas.csv");
        let parquet: std::path::PathBuf = temp_dir.join("test_unified_empresas.parquet");

        let mut file: File = File::create(&csv).expect("Create temp csv");
        file.write_all(b"\"12ABC345\";\"EMPRESA ALFA\";\"2062\";\"49\";\"15000,00\";\"03\";\"\"\n")
            .expect("Write csv");
        drop(file);

        let rows: usize = to_parquet(&csv, &parquet, TableKind::Empresas).expect("Convert");
        assert_eq!(rows, 1);
        assert!(parquet.exists());

        let _ = std::fs::remove_file(csv);
        let _ = std::fs::remove_file(parquet);
    }

    #[test]
    fn test_unified_to_parquet_socios() {
        let temp_dir: std::path::PathBuf = std::env::temp_dir();
        let csv: std::path::PathBuf = temp_dir.join("test_unified_socios.csv");
        let parquet: std::path::PathBuf = temp_dir.join("test_unified_socios.parquet");

        let mut file: File = File::create(&csv).expect("Create temp csv");
        file.write_all(b"\"12ABC345\";\"2\";\"MARIA SILVA\";\"***123456**\";\"49\";\"20200115\";\"\";\"***000000**\";\"JOSE SILVA\";\"05\";\"5\"\n")
            .expect("Write csv");
        drop(file);

        let rows: usize = to_parquet(&csv, &parquet, TableKind::Socios).expect("Convert");
        assert_eq!(rows, 1);
        assert!(parquet.exists());

        let _ = std::fs::remove_file(csv);
        let _ = std::fs::remove_file(parquet);
    }

    #[test]
    fn test_unified_to_parquet_estabelecimentos() {
        let temp_dir: std::path::PathBuf = std::env::temp_dir();
        let csv: std::path::PathBuf = temp_dir.join("test_unified_estab.csv");
        let parquet: std::path::PathBuf = temp_dir.join("test_unified_estab.parquet");

        let mut file: File = File::create(&csv).expect("Create temp csv");
        file.write_all(b"\"12ABC345\";\"0001\";\"95\";\"1\";\"MATRIZ\";\"02\";\"20210510\";\"00\";\"\";\"\";\"20210510\";\"6201501\";\"\";\"AVENIDA\";\"PAULISTA\";\"1000\";\"SALA 10\";\"BELA VISTA\";\"01310100\";\"SP\";\"7107\";\"11\";\"33334444\";\"\";\"\";\"\";\"\";\"contato@empresa.com\";\"\";\"\"\n")
            .expect("Write csv");
        drop(file);

        let rows: usize = to_parquet(&csv, &parquet, TableKind::Estabelecimentos).expect("Convert");
        assert_eq!(rows, 1);
        assert!(parquet.exists());

        let _ = std::fs::remove_file(csv);
        let _ = std::fs::remove_file(parquet);
    }

    #[test]
    fn test_unified_to_parquet_simples() {
        let temp_dir: std::path::PathBuf = std::env::temp_dir();
        let csv: std::path::PathBuf = temp_dir.join("test_unified_simples.csv");
        let parquet: std::path::PathBuf = temp_dir.join("test_unified_simples.parquet");

        let mut file: File = File::create(&csv).expect("Create temp csv");
        file.write_all(b"\"12ABC345\";\"S\";\"20200101\";\"20211231\";\"N\";\"\";\"\"\n")
            .expect("Write csv");
        drop(file);

        let rows: usize = to_parquet(&csv, &parquet, TableKind::Simples).expect("Convert");
        assert_eq!(rows, 1);
        assert!(parquet.exists());

        let _ = std::fs::remove_file(csv);
        let _ = std::fs::remove_file(parquet);
    }

    #[test]
    fn test_unified_to_parquet_cnaes() {
        let temp_dir: std::path::PathBuf = std::env::temp_dir();
        let csv: std::path::PathBuf = temp_dir.join("test_unified_cnaes.csv");
        let parquet: std::path::PathBuf = temp_dir.join("test_unified_cnaes.parquet");

        let mut file: File = File::create(&csv).expect("Create temp csv");
        file.write_all(b"\"6201501\";\"DESENVOLVIMENTO DE PROGRAMAS\"\n")
            .expect("Write csv");
        drop(file);

        let rows: usize = to_parquet(&csv, &parquet, TableKind::Cnaes).expect("Convert");
        assert_eq!(rows, 1);
        assert!(parquet.exists());

        let _ = std::fs::remove_file(csv);
        let _ = std::fs::remove_file(parquet);
    }

    #[test]
    fn test_unified_to_parquet_municipios() {
        let temp_dir: std::path::PathBuf = std::env::temp_dir();
        let csv: std::path::PathBuf = temp_dir.join("test_unified_municipios.csv");
        let parquet: std::path::PathBuf = temp_dir.join("test_unified_municipios.parquet");

        let mut file: File = File::create(&csv).expect("Create temp csv");
        file.write_all(b"\"7107\";\"SAO PAULO\"\n")
            .expect("Write csv");
        drop(file);

        let rows: usize = to_parquet(&csv, &parquet, TableKind::Municipios).expect("Convert");
        assert_eq!(rows, 1);
        assert!(parquet.exists());

        let _ = std::fs::remove_file(csv);
        let _ = std::fs::remove_file(parquet);
    }
}
