# Benchmarks historicos

Resultados de benchmarks do pipeline CNPydge. Colar aqui resultados relevantes apos cada
mudanca significativa de performance.

## Formato de registro

```markdown
## [DATA] Descricao da mudanca

- **Arquivo**: tamanho do arquivo de teste
- **Throughput**: X registros/s, Y MB/s
- **RAM**: Z MB (pico)
- **Ambiente**: CPU, disco, SO
- **Commit**: hash do commit testado
```

## Notas

- Nao colar logs brutos -- resumir metricas relevantes.
- Linkar para o commit ou branch quando possivel.
- Manter apenas resultados de mudancas que impactaram performance de forma mensuravel.
