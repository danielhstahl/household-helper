from fastapi import HTTPException, Depends, status
from fastapi.security import OAuth2PasswordBearer
from sqlalchemy.orm import Session
from typing import Optional
from datetime import datetime, timedelta, timezone
from jose import JWTError, jwt
import secrets
from models import User, UserInDB, CurrentUser, TokenData, UserCreate, UserUpdate, Roles
from typing import Iterator
from sqlalchemy import create_engine, delete
from sqlalchemy.orm import sessionmaker
import bcrypt
import os

SECRET_KEY = secrets.token_hex(
    32
)  # if server restarts this will be awkward, pass in as an env variable instead
ALGORITHM = "HS256"


# Hash a password using bcrypt
def hash_password(password: str) -> str:
    pwd_bytes = password.encode("utf-8")
    salt = bcrypt.gensalt()
    hashed_password = bcrypt.hashpw(password=pwd_bytes, salt=salt)
    return hashed_password.decode("utf-8")


# Check if the provided password matches the stored password (hashed)
def verify_password(plain_password: str, hashed_password: str):
    password_byte_enc = plain_password.encode("utf-8")
    return bcrypt.checkpw(
        password=password_byte_enc, hashed_password=hashed_password.encode("utf-8")
    )


# --- OAuth2PasswordBearer is used to extract the token from the Authorization header ---
oauth2_scheme = OAuth2PasswordBearer(tokenUrl="token")

DATABASE_URL = os.getenv(
    "DATABASE_URL",
    "sqlite://",  # in memory
)  # "postgresql://postgres:yourpassword@localhost:5432/fastapi_db"
engine = create_engine(DATABASE_URL)
SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)


# --- Database Session Dependency ---
def get_db() -> Iterator[Session]:
    """Dependency to provide a database session."""
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()


async def authenticate_user(
    db: Session, username: str = "", password: str = ""
) -> Optional[UserInDB]:
    """Authenticates a user by checking username and password."""
    user = get_user_from_db(db, username)
    if not user:
        return None
    if not verify_password(password, user.hashed_password):
        return None
    return user


async def get_current_user(
    db: Session = Depends(get_db), token: str = Depends(oauth2_scheme)
) -> CurrentUser:
    """Dependency to get the currently authenticated user based on a JWT token."""
    credentials_exception = HTTPException(
        status_code=status.HTTP_401_UNAUTHORIZED,
        detail="Could not validate credentials",
        headers={"WWW-Authenticate": "Bearer"},
    )
    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=[ALGORITHM])
        username: Optional[str] = payload.get("sub")
        if username is None:
            raise credentials_exception
        token_data = TokenData(username=username)
    except JWTError:
        raise credentials_exception

    user = get_user_from_db(db, token_data.username or "")
    if user is None:
        raise credentials_exception  # User not found in DB

    return CurrentUser(
        id=user.id,
        username=user.username,
        disabled=user.disabled,
        roles=[role.role for role in user.roles],
    )


async def get_current_admin_user(
    current_user: CurrentUser = Depends(get_current_user),
) -> CurrentUser:
    """Dependency to ensure the current user is an administrator."""
    if "admin" not in current_user.roles:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail="Not enough permissions to perform this action",
        )
    return current_user


def get_current_user_by_roles(required_role: str):
    async def get_current_user_by_roles(
        current_user: CurrentUser = Depends(get_current_user),
    ) -> CurrentUser:
        """Dependency to ensure the current user has access to the role."""
        if required_role not in current_user.roles:
            raise HTTPException(
                status_code=status.HTTP_403_FORBIDDEN,
                detail="Not enough permissions to perform this action",
            )
        return current_user

    return get_current_user_by_roles


def get_user_from_db(db: Session, username: str) -> Optional[UserInDB]:
    """Retrieves a user from the database by username."""
    return db.query(User).filter(User.username == username).first()


def get_user_from_db_by_id(db: Session, id: int) -> Optional[UserInDB]:
    """Retrieves a user from the database by id."""
    return db.get(User, id)


def create_user_in_db_func(db: Session, user: UserCreate) -> User:
    """Creates a new user and adds them to the database."""
    hashed_password = hash_password(user.password)
    db_user = User(
        username=user.username,
        hashed_password=hashed_password,
        roles=[Roles(role=role) for role in user.roles],
    )
    db.add(db_user)
    # for role in user.roles:
    #    db_role = Roles(username_id=db_user.id, role=role, user=db_user)
    #    db.add(db_role)
    db.commit()
    db.refresh(db_user)  # Refresh to get the generated ID
    return db_user


def delete_user_in_db_func(db: Session, db_user: UserInDB) -> UserInDB:
    """Creates a new user and adds them to the database."""
    db.delete(db_user)
    db.commit()
    return db_user


def update_user_in_db_func(
    db: Session, db_user: UserInDB, user: UserUpdate
) -> UserInDB:
    delete_stmt = delete(Roles).where(Roles.username_id == db_user.id)
    db.execute(delete_stmt)
    db_user.hashed_password = (
        hash_password(user.password) if user.password else db_user.hashed_password
    )
    db_user.roles = [Roles(role=role, username_id=db_user.id) for role in user.roles]
    db.merge(db_user)
    db.commit()
    db.refresh(db_user)  # Refresh roles
    return db_user


# --- JWT Token Generation ---
def create_access_token(data: dict, expires_delta: Optional[timedelta] = None):
    """Creates a JWT access token."""
    to_encode = data.copy()
    if expires_delta:
        expire = datetime.now(timezone.utc) + expires_delta
    else:
        expire = datetime.now(timezone.utc) + timedelta(minutes=15)
    to_encode.update({"exp": expire})
    encoded_jwt = jwt.encode(to_encode, SECRET_KEY, algorithm=ALGORITHM)
    return encoded_jwt
