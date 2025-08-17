from pydantic import BaseModel, Field
from typing import Optional
from sqlalchemy.orm import sessionmaker, declarative_base, mapped_column, relationship
from sqlalchemy import (
    create_engine,
    Column,
    Integer,
    String,
    Boolean,
    UniqueConstraint,
    ForeignKey,
)
from passlib.context import CryptContext
from sqlalchemy.orm import Session
from datetime import datetime, timedelta
from jose import JWTError, jwt
import secrets

SECRET_KEY = secrets.token_hex(
    32
)  # if server restarts this will be awkward, pass in as an env variable instead
ALGORITHM = "HS256"
ACCESS_TOKEN_EXPIRE_MINUTES = 30  # Token valid for 30 minutes
Base = declarative_base()
# Password hashing context
pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")


class User(Base):
    """SQLAlchemy model for the 'users' table."""

    __tablename__ = "users"
    username = mapped_column(String, primary_key=True, index=True)
    hashed_password = mapped_column(String, nullable=False)
    disabled = mapped_column(Boolean, default=False)
    roles = relationship("Roles", back_populates="role")


class Roles(Base):
    """SQLAlchemy model for the 'roles' table."""

    __tablename__ = "roles"

    id = mapped_column(Integer, primary_key=True, index=True)
    username = mapped_column(ForeignKey("users.username"))
    role = mapped_column(String, nullable=False)
    __table_args__ = (UniqueConstraint("username", "role", name="_username_role"),)


# --- Pydantic Models ---


class UserInDB(BaseModel):
    """Internal model for a user retrieved from the database."""

    id: int
    username: str
    hashed_password: str
    disabled: Optional[bool] = None
    roles: list[str]

    class Config:
        from_attributes = True  # Updated from orm_mode = True for Pydantic v2


class UserCreate(BaseModel):
    """Model for creating a new user (request body)."""

    username: str = Field(..., min_length=3, max_length=50)
    password: str = Field(..., min_length=6)
    roles: list[str]
    # is_admin: bool = False  # Allow setting admin status on creation (admin-only)


class UserLogin(BaseModel):
    """Model for user login requests."""

    username: str
    password: str


class Token(BaseModel):
    """Model for the authentication token response."""

    access_token: str
    token_type: str = "bearer"


class TokenData(BaseModel):
    """Model to store data extracted from the JWT."""

    username: Optional[str] = None


class CurrentUser(BaseModel):
    """Model for the currently authenticated user (for response)."""

    username: str
    disabled: Optional[bool] = None
    roles: list[str]
