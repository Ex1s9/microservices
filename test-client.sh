#!/bin/bash

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${GREEN}=== GameHub Services Test Client ===${NC}"

# Проверяем, установлен ли grpcurl
if ! command -v grpcurl &> /dev/null; then
    echo -e "${RED}grpcurl is not installed!${NC}"
    echo -e "${YELLOW}Install with:${NC}"
    echo "  macOS: brew install grpcurl"
    echo "  Linux: go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest"
    exit 1
fi

USER_SERVICE="localhost:50051"
GAME_SERVICE="localhost:50052"

echo -e "${YELLOW}Available tests:${NC}"
echo "1) Test User Service - Create User"
echo "2) Test User Service - List Users"
echo "3) Test Game Service - Create Game"
echo "4) Show User Service methods"
echo "5) Show Game Service methods"
echo "6) Exit"

read -p "Enter your choice (1-6): " choice

case $choice in
    1)
        echo -e "${BLUE}Testing User Service - Create User${NC}"
        grpcurl -plaintext -d '{
            "email": "test@example.com",
            "username": "testuser",
            "password": "password123",
            "role": 0
        }' $USER_SERVICE user.UserService/CreateUser
        ;;
    2)
        echo -e "${BLUE}Testing User Service - List Users${NC}"
        grpcurl -plaintext -d '{
            "limit": 10,
            "offset": 0
        }' $USER_SERVICE user.UserService/ListUsers
        ;;
    3)
        echo -e "${BLUE}Testing Game Service - Create Game${NC}"
        grpcurl -plaintext -d '{
            "name": "Test Game",
            "description": "A test game for demonstration",
            "developer_id": "550e8400-e29b-41d4-a716-446655440000",
            "release_date": "2024-01-01",
            "categories": [0],
            "tags": ["test", "demo"],
            "platforms": ["PC", "Mac"],
            "price": 29.99
        }' $GAME_SERVICE game.GameService/CreateGame
        ;;
    4)
        echo -e "${BLUE}User Service Methods:${NC}"
        grpcurl -plaintext $USER_SERVICE list user.UserService
        ;;
    5)
        echo -e "${BLUE}Game Service Methods:${NC}"
        grpcurl -plaintext $GAME_SERVICE list game.GameService
        ;;
    6)
        echo "Exiting..."
        exit 0
        ;;
    *)
        echo -e "${RED}Invalid choice${NC}"
        exit 1
        ;;
esac