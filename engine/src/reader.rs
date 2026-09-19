//! Módulo de leitura de alta vazão com mapeamento de memória e particionamento em blocos.

use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

/// Gerencia a leitura de arquivos massivos via mapeamento de memória (`mmap`).
///
/// Delega ao kernel do sistema operacional a paginação sob demanda dos blocos
/// do disco, evitando alocações excessivas na memória RAM.
pub struct MmapReader {
    mmap: Mmap,
}

impl MmapReader {
    /// Abre um arquivo e mapeia seu conteúdo integral no espaço de memória virtual.
    ///
    /// # Parâmetros
    /// - `path`: Caminho para o arquivo no sistema de arquivos.
    ///
    /// # Erros
    /// Retorna erro de I/O caso o arquivo não exista, não possa ser lido ou ocorra falha no mapeamento.
    ///
    /// # Segurança
    /// O mapeamento de memória é considerado `unsafe` na biblioteca `memmap2` porque alterações
    /// concorrentes no arquivo por outros processos podem causar comportamento indefinido.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        // SAFETY: Mapeamento de arquivo somente-leitura destinado a processamento de batches imutáveis.
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Self { mmap })
    }

    /// Retorna a referência direta à fatia contígua de bytes mapeados.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.mmap
    }

    /// Retorna o tamanho total do arquivo em bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.mmap.len()
    }

    /// Indica se o arquivo mapeado está vazio.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.mmap.is_empty()
    }

    /// Calcula os limites `(início, fim)` para dividir o arquivo em blocos alinhados por quebras de linha.
    ///
    /// Garante custo operacional mínimo (zero-copy) retornando apenas índices de corte sem alocar buffers.
    pub fn chunk_boundaries(&self, num_chunks: usize) -> Vec<(usize, usize)> {
        find_chunk_boundaries(&self.mmap, num_chunks)
    }
}

/// Localiza as fronteiras de corte em um buffer de bytes alinhando cada bloco com a quebra de linha (`\n`).
///
/// # Invariantes Garantidos
/// 1. Continuidade: O início do bloco `i + 1` é exatamente igual ao fim do bloco `i`.
/// 2. Integridade de registros: Nenhuma linha é cortada ao meio; os blocos intermediários terminam em `\n`.
/// 3. Cobertura total: A soma dos comprimentos de todos os blocos equivale exatamente ao tamanho total do buffer.
pub fn find_chunk_boundaries(data: &[u8], num_chunks: usize) -> Vec<(usize, usize)> {
    if data.is_empty() || num_chunks == 0 {
        return Vec::new();
    }

    if num_chunks == 1 || data.len() <= num_chunks {
        return vec![(0, data.len())];
    }

    let chunk_size = data.len() / num_chunks;
    let mut boundaries = Vec::with_capacity(num_chunks);
    let mut start = 0;

    while start < data.len() {
        if boundaries.len() + 1 == num_chunks {
            // Último bloco absorve todo o restante do arquivo
            boundaries.push((start, data.len()));
            break;
        }

        let tentative_end = start + chunk_size;
        if tentative_end >= data.len() {
            boundaries.push((start, data.len()));
            break;
        }

        // Busca a próxima quebra de linha a partir do ponto pretendido de corte
        let next_newline = data[tentative_end..]
            .iter()
            .position(|&b| b == b'\n')
            .map(|pos| tentative_end + pos + 1)
            .unwrap_or(data.len());

        boundaries.push((start, next_newline));
        start = next_newline;
    }

    boundaries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_buffer() {
        let boundaries: Vec<(usize, usize)> = find_chunk_boundaries(b"", 4);
        assert!(boundaries.is_empty());
    }

    #[test]
    fn test_zero_chunks() {
        let boundaries: Vec<(usize, usize)> = find_chunk_boundaries(b"linha 1\nlinha 2\n", 0);
        assert!(boundaries.is_empty());
    }

    #[test]
    fn test_single_chunk() {
        let data: &[u8] = b"primeira linha\nsegunda linha\n";
        let boundaries: Vec<(usize, usize)> = find_chunk_boundaries(data, 1);
        assert_eq!(boundaries, vec![(0, data.len())]);
    }

    #[test]
    fn test_chunk_boundaries_preserve_complete_lines() {
        let data: &[u8] = b"12345678000195;EMPRESA A;MATRIZ\n98765432000100;EMPRESA B;FILIAL\n11223344000155;EMPRESA C;MATRIZ\n44332211000188;EMPRESA D;FILIAL\n";
        let boundaries: Vec<(usize, usize)> = find_chunk_boundaries(data, 2);

        assert_eq!(boundaries.len(), 2);

        // Verifica continuidade
        assert_eq!(boundaries[0].0, 0);
        assert_eq!(boundaries[0].1, boundaries[1].0);
        assert_eq!(boundaries[1].1, data.len());

        // Cada fatia intermediária deve terminar exatamente com '\n'
        assert_eq!(data[boundaries[0].1 - 1], b'\n');

        // A soma dos blocos deve cobrir 100% dos bytes
        let total_bytes: usize = boundaries.iter().map(|(s, e)| e - s).sum();
        assert_eq!(total_bytes, data.len());
    }

    #[test]
    fn test_file_without_trailing_newline() {
        let data: &[u8] = b"linha 1\nlinha 2\nlinha 3 sem newline";
        let boundaries: Vec<(usize, usize)> = find_chunk_boundaries(data, 3);

        assert!(!boundaries.is_empty());
        let total_bytes: usize = boundaries.iter().map(|(s, e)| e - s).sum();
        assert_eq!(total_bytes, data.len());
        assert_eq!(boundaries.last().unwrap().1, data.len());
    }
}
