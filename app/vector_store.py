from llama_index.core.base.embeddings.base import BaseEmbedding
from sqlalchemy import make_url
from llama_index.core import VectorStoreIndex
from llama_index.vector_stores.postgres import PGVectorStore
from llama_index.core.vector_stores.types import BasePydanticVectorStore
import os


def get_connection_information() -> tuple[str, str]:
    VECTOR_DATABASE_URL = os.getenv(
        "VECTOR_DATABASE_URL", "postgresql://postgres:yourpassword@localhost:5432"
    )
    db_name = "vector_db"
    return VECTOR_DATABASE_URL, db_name


def get_vector_store(
    connection_string: str,
    db_name: str,
    table_name: str,
) -> BasePydanticVectorStore:
    url = make_url(connection_string)
    return PGVectorStore.from_params(
        database=db_name,
        host=url.host,
        password=url.password,
        port=str(url.port),
        user=url.username,
        table_name=table_name,
        embed_dim=1024,  # bge-m3 embedding dimension
        hnsw_kwargs={
            "hnsw_m": 16,
            "hnsw_ef_construction": 64,
            "hnsw_ef_search": 40,
            "hnsw_dist_method": "vector_cosine_ops",
        },
    )


# unused for now
def get_vector_store_index(
    vector_store: BasePydanticVectorStore,
    embedding_model: BaseEmbedding,
) -> VectorStoreIndex:
    return VectorStoreIndex.from_vector_store(vector_store, embedding_model)
