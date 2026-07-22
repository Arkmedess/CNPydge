"""API REST de consulta (FastAPI). Ver skills: api-designer, sql-optimizer.

Somente leitura -- nunca aciona o pipeline de importacao via HTTP.
"""

from fastapi import FastAPI

app = FastAPI(title="CNPJ Ultra Importer API")
# TODO: Fase 7 do roadmap.
