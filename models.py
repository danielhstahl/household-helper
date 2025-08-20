from pydantic import BaseModel, Field
from typing import Optional
from sqlalchemy.orm import declarative_base, mapped_column, relationship
from sqlalchemy import Integer, String, Boolean, UniqueConstraint, ForeignKey, DateTime
from datetime import datetime, timezone

Base = declarative_base()


class User(Base):
    """SQLAlchemy model for the 'users' table."""

    __tablename__ = "users"
    id = mapped_column(Integer, primary_key=True, index=True)
    username = mapped_column(String, unique=True, index=True)
    hashed_password = mapped_column(String, nullable=False)
    disabled = mapped_column(Boolean, default=False)
    roles = relationship("Roles", back_populates="user", cascade="all, delete")
    # cascade = (
    #    "all, delete-orphan"  # delete everything that FKs to ID when deleting a user
    # )


class Roles(Base):
    """SQLAlchemy model for the 'roles' table."""

    __tablename__ = "roles"

    id = mapped_column(Integer, primary_key=True, index=True)
    username_id = mapped_column(
        Integer, ForeignKey("users.id", ondelete="CASCADE"), nullable=False
    )
    role = mapped_column(String, nullable=False)
    user = relationship("User", back_populates="roles")
    __table_args__ = (UniqueConstraint("username_id", "role", name="_username_role"),)


class Sessions(Base):
    """SQLAlchemy model for individual chat messages."""

    __tablename__ = "sessions"
    id = mapped_column(String, primary_key=True, index=True)
    username_id = mapped_column(
        Integer, ForeignKey("users.id", ondelete="CASCADE"), nullable=False
    )
    # cascade = (
    #    "all, delete-orphan"  # delete everything that FKs to ID when deleting a session
    # )


class Message(Base):
    """SQLAlchemy model for individual chat messages."""

    __tablename__ = "chat_messages"

    id = mapped_column(Integer, primary_key=True, index=True)
    session_id = mapped_column(
        String, ForeignKey("sessions.id", ondelete="CASCADE"), nullable=False
    )
    username_id = mapped_column(
        Integer, ForeignKey("users.id", ondelete="CASCADE"), nullable=False
    )
    content = mapped_column(String, nullable=False)
    timestamp = mapped_column(
        DateTime, default=datetime.now(timezone.utc), nullable=False
    )


# --- Pydantic Models ---


class RoleInDB(BaseModel):
    id: int
    username: str
    role: str


# Is this actually necesary??
class UserInDB(BaseModel):
    """Internal model for a user retrieved from the database."""

    id: int
    username: str
    hashed_password: str
    disabled: Optional[bool] = None
    roles: list[RoleInDB]

    class Config:
        from_attributes = True  # Updated from orm_mode = True for Pydantic v2


class UserCreate(BaseModel):
    """Model for creating a new user (request body)."""

    username: str = Field(..., min_length=3, max_length=50)
    password: str = Field(..., min_length=6)
    roles: list[str]
    # is_admin: bool = False  # Allow setting admin status on creation (admin-only)


class UserUpdate(BaseModel):
    """Model for updating a new user (request body)."""

    id: int
    username: str = Field(..., min_length=3, max_length=50)
    password: Optional[str] = Field(..., min_length=6)
    roles: list[str]


class UserDelete(BaseModel):
    """Model for updating a new user (request body)."""

    id: int


class UserLogin(BaseModel):
    """Model for user login requests."""

    username: str
    password: str


class Token(BaseModel):
    """Model for the authentication token response."""

    access_token: str
    token_type: str = "bearer"


class GenericSuccess(BaseModel):
    """Model for generic response."""

    status: str


class TokenData(BaseModel):
    """Model to store data extracted from the JWT."""

    username: Optional[str] = None


class CurrentUser(BaseModel):
    """Model for the currently authenticated user (for response)."""

    id: int
    username: str
    disabled: Optional[bool] = None
    roles: list[str]


class SessionAndUser(BaseModel):
    sessions: list[str]
    user: CurrentUser


# Add this to your User model if you want a bidirectional relationship
# class User(...):
#    ...
#    chat_messages = relationship("ChatMessage", back_populates="user", order_by=ChatMessage.timestamp)
