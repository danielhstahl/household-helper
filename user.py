from fastapi import HTTPException, Depends, status
from fastapi.security import OAuth2PasswordBearer
from sqlalchemy.orm import Session
from typing import Optional
from datetime import datetime, timedelta
from passlib.context import CryptContext
from jose import JWTError, jwt
import secrets
from models import User, UserInDB, CurrentUser, TokenData, UserCreate

SECRET_KEY = secrets.token_hex(
    32
)  # if server restarts this will be awkward, pass in as an env variable instead
ALGORITHM = "HS256"


pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")

# --- OAuth2PasswordBearer is used to extract the token from the Authorization header ---
oauth2_scheme = OAuth2PasswordBearer(tokenUrl="token")


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
    db: Session, token: str = Depends(oauth2_scheme)
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
        username=user.username,
        disabled=user.disabled,
        roles=user.roles,
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


def hash_password(password: str) -> str:
    """Hashes a plain-text password."""
    return pwd_context.hash(password)


def verify_password(plain_password: str, hashed_password: str) -> bool:
    """Verifies a plain-text password against a hashed password."""
    return pwd_context.verify(plain_password, hashed_password)


def get_user_from_db(db: Session, username: str) -> Optional[UserInDB]:
    """Retrieves a user from the database by username."""
    return db.query(User).filter(User.username == username).first()


def create_user_in_db_func(db: Session, user: UserCreate) -> UserInDB:
    """Creates a new user and adds them to the database."""
    hashed_password = hash_password(user.password)
    db_user = User(
        username=user.username,
        hashed_password=hashed_password,
        roles=user.roles,
    )
    db.add(db_user)
    db.commit()
    # db.refresh(db_user)  # Refresh to get the generated ID
    return db_user


def update_user_in_db_func(db: Session, user: UserCreate) -> UserInDB:
    """Creates a new user and adds them to the database."""
    hashed_password = hash_password(user.password)
    db_user = User(
        username=user.username,
        hashed_password=hashed_password,
        roles=user.roles,
    )
    db.merge(db_user)
    db.commit()
    return db_user


# --- JWT Token Generation ---
def create_access_token(data: dict, expires_delta: Optional[timedelta] = None):
    """Creates a JWT access token."""
    to_encode = data.copy()
    if expires_delta:
        expire = datetime.utcnow() + expires_delta
    else:
        expire = datetime.utcnow() + timedelta(minutes=15)
    to_encode.update({"exp": expire})
    encoded_jwt = jwt.encode(to_encode, SECRET_KEY, algorithm=ALGORITHM)
    return encoded_jwt
