from llama_index.core.embeddings.utils import EmbedType
from sqlalchemy import make_url
from llama_index.core import VectorStoreIndex
from llama_index.vector_stores.postgres import PGVectorStore
from llama_index.core import Document


def get_connection_information() -> tuple[str, str]:
    connection_string = "postgresql://postgres:yourpassword@localhost:5432"
    db_name = "vector_db"
    return connection_string, db_name


def get_vector_store_index(
    connection_string: str, db_name: str, table_name: str, embedding_model: EmbedType
) -> VectorStoreIndex:
    url = make_url(connection_string)
    pg_vector_store = PGVectorStore.from_params(
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
    return VectorStoreIndex.from_vector_store(pg_vector_store, embedding_model)


def add_to_store(vector_store_index: VectorStoreIndex, text: str):
    # do I need to tokenize first?
    vector_store_index.insert(Document(text_resource={"text": text}))


# UNUSED
def retrieve_from_store(vector_store_index: VectorStoreIndex, text: str) -> list[str]:
    base_retriever = vector_store_index.as_retriever(similarity_top_k=5)
    return [node.get_content() for node in base_retriever.retrieve(text)]
