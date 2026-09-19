//! Representação e validação com tipagem forte dos registros da tabela de Estabelecimentos do CNPJ.

use crate::util::{
    ErroCampo, clean_quotes, opt_str, parse_opt_u16, parse_opt_u32, parse_u8, parse_u16, parse_u32,
};

/// Registro validado e fortemente tipado de um Estabelecimento da Receita Federal.
#[derive(Debug, PartialEq)]
pub struct Estabelecimento<'a> {
    pub cnpj_basico: &'a [u8],               // 8 caracteres alfanuméricos
    pub cnpj_ordem: &'a [u8],                // 4 dígitos (ex.: "0001")
    pub cnpj_dv: &'a [u8],                   // 2 dígitos verificadores
    pub matriz_filial: u8,                   // 1 = Matriz, 2 = Filial
    pub fantasia: Option<&'a str>,           // Nome fantasia
    pub situacao: u8,                        // Situação cadastral
    pub data_situacao: Option<u32>,          // AAAAMMDD
    pub motivo_situacao: u16,                // Código do motivo da situação
    pub cidade_exterior: Option<&'a str>,    // Nome da cidade no exterior
    pub pais: Option<u16>,                   // Código do país
    pub data_inicio: Option<u32>,            // Data de início da atividade
    pub cnae_principal: u32,                 // Código CNAE fiscal principal (7 dígitos)
    pub cnae_secundario: Option<&'a str>,    // Lista de CNAEs secundários
    pub tipo_logradouro: Option<&'a str>,    // Tipo de logradouro
    pub logradouro: Option<&'a str>,         // Logradouro
    pub numero: Option<&'a str>,             // Número
    pub complemento: Option<&'a str>,        // Complemento
    pub bairro: Option<&'a str>,             // Bairro
    pub cep: Option<&'a str>,                // CEP
    pub uf: Option<&'a str>,                 // Unidade federativa (sigla estado)
    pub municipio: Option<u16>,              // Código do município (TOM)
    pub ddd_1: Option<&'a str>,              // DDD 1
    pub telefone_1: Option<&'a str>,         // Telefone 1
    pub ddd_2: Option<&'a str>,              // DDD 2
    pub telefone_2: Option<&'a str>,         // Telefone 2
    pub ddd_fax: Option<&'a str>,            // DDD Fax
    pub fax: Option<&'a str>,                // Fax
    pub email: Option<&'a str>,              // E-mail
    pub situacao_especial: Option<&'a str>,  // Situação especial
    pub data_situacao_especial: Option<u32>, // Data da situação especial
}

impl<'a> Estabelecimento<'a> {
    /// Faz o parsing e a validação estrita de uma linha CSV de Estabelecimento.
    pub fn parse_line(line: &'a [u8]) -> Result<Self, ErroCampo> {
        let mut campos: [&'a [u8]; 30] = [&[]; 30];
        let mut indice_campo: usize = 0;
        let mut inicio: usize = 0;
        let mut em_aspas: bool = false;

        for (i, &byte) in line.iter().enumerate() {
            if byte == b'"' {
                em_aspas = !em_aspas;
            } else if byte == b';' && !em_aspas {
                if indice_campo < 30 {
                    campos[indice_campo] = &line[inicio..i];
                    indice_campo += 1;
                }
                inicio = i + 1;
            }
        }

        if indice_campo < 30 && inicio <= line.len() {
            let mut fim: usize = line.len();
            while fim > inicio && (line[fim - 1] == b'\n' || line[fim - 1] == b'\r') {
                fim -= 1;
            }
            campos[indice_campo] = &line[inicio..fim];
            indice_campo += 1;
        }

        if indice_campo != 30 {
            return Err(ErroCampo::CamposInsuficientes);
        }

        let cnpj_basico: &'a [u8] = clean_quotes(campos[0]);
        if cnpj_basico.len() != 8 || !cnpj_basico.iter().all(|b: &u8| b.is_ascii_alphanumeric()) {
            return Err(ErroCampo::CnpjInvalido);
        }

        let cnpj_ordem: &'a [u8] = clean_quotes(campos[1]);
        let cnpj_dv: &'a [u8] = clean_quotes(campos[2]);
        let matriz_filial: u8 = parse_u8(clean_quotes(campos[3]))?;
        let fantasia: Option<&'a str> = opt_str(clean_quotes(campos[4]))?;
        let situacao: u8 = parse_u8(clean_quotes(campos[5]))?;
        let data_situacao: Option<u32> = parse_opt_u32(clean_quotes(campos[6]))?;
        let motivo_situacao: u16 = parse_u16(clean_quotes(campos[7]))?;
        let cidade_exterior: Option<&'a str> = opt_str(clean_quotes(campos[8]))?;
        let pais: Option<u16> = parse_opt_u16(clean_quotes(campos[9]))?;
        let data_inicio: Option<u32> = parse_opt_u32(clean_quotes(campos[10]))?;
        let cnae_principal: u32 = parse_u32(clean_quotes(campos[11]))?;
        let cnae_secundario: Option<&'a str> = opt_str(clean_quotes(campos[12]))?;
        let tipo_logradouro: Option<&'a str> = opt_str(clean_quotes(campos[13]))?;
        let logradouro: Option<&'a str> = opt_str(clean_quotes(campos[14]))?;
        let numero: Option<&'a str> = opt_str(clean_quotes(campos[15]))?;
        let complemento: Option<&'a str> = opt_str(clean_quotes(campos[16]))?;
        let bairro: Option<&'a str> = opt_str(clean_quotes(campos[17]))?;
        let cep: Option<&'a str> = opt_str(clean_quotes(campos[18]))?;
        let uf: Option<&'a str> = opt_str(clean_quotes(campos[19]))?;
        let municipio: Option<u16> = parse_opt_u16(clean_quotes(campos[20]))?;
        let ddd_1: Option<&'a str> = opt_str(clean_quotes(campos[21]))?;
        let telefone_1: Option<&'a str> = opt_str(clean_quotes(campos[22]))?;
        let ddd_2: Option<&'a str> = opt_str(clean_quotes(campos[23]))?;
        let telefone_2: Option<&'a str> = opt_str(clean_quotes(campos[24]))?;
        let ddd_fax: Option<&'a str> = opt_str(clean_quotes(campos[25]))?;
        let fax: Option<&'a str> = opt_str(clean_quotes(campos[26]))?;
        let email: Option<&'a str> = opt_str(clean_quotes(campos[27]))?;
        let situacao_especial: Option<&'a str> = opt_str(clean_quotes(campos[28]))?;
        let data_situacao_especial: Option<u32> = parse_opt_u32(clean_quotes(campos[29]))?;

        Ok(Self {
            cnpj_basico,
            cnpj_ordem,
            cnpj_dv,
            matriz_filial,
            fantasia,
            situacao,
            data_situacao,
            motivo_situacao,
            cidade_exterior,
            pais,
            data_inicio,
            cnae_principal,
            cnae_secundario,
            tipo_logradouro,
            logradouro,
            numero,
            complemento,
            bairro,
            cep,
            uf,
            municipio,
            ddd_1,
            telefone_1,
            ddd_2,
            telefone_2,
            ddd_fax,
            fax,
            email,
            situacao_especial,
            data_situacao_especial,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_estabelecimento_line() {
        let line: &[u8] = b"\"12ABC345\";\"0001\";\"95\";\"1\";\"FANTASIA MATRIZ\";\"02\";\"20210510\";\"00\";\"\";\"\";\"20210510\";\"6201501\";\"\";\"AVENIDA\";\"PAULISTA\";\"1000\";\"SALA 10\";\"BELA VISTA\";\"01310100\";\"SP\";\"7107\";\"11\";\"33334444\";\"\";\"\";\"\";\"\";\"contato@empresa.com\";\"\";\"\"";
        let est: Estabelecimento<'_> =
            Estabelecimento::parse_line(line).expect("Valid estabelecimento line");

        assert_eq!(est.cnpj_basico, b"12ABC345");
        assert_eq!(est.cnpj_ordem, b"0001");
        assert_eq!(est.cnpj_dv, b"95");
        assert_eq!(est.matriz_filial, 1);
        assert_eq!(est.fantasia, Some("FANTASIA MATRIZ"));
        assert_eq!(est.situacao, 2);
        assert_eq!(est.cnae_principal, 6201501);
        assert_eq!(est.uf, Some("SP"));
        assert_eq!(est.municipio, Some(7107));
    }

    #[test]
    fn test_estabelecimento_insufficient_fields() {
        let line: &[u8] = b"\"12ABC345\";\"0001\";\"95\";\"1\"";
        let result: Result<Estabelecimento<'_>, ErroCampo> = Estabelecimento::parse_line(line);
        assert_eq!(result, Err(ErroCampo::CamposInsuficientes));
    }
}
