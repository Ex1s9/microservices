RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== GameHub Services Launcher ===${NC}"

# Проверка и запуск PostgreSQL
echo -e "${YELLOW}Checking PostgreSQL...${NC}"
if ! docker ps | grep -q gamehub_postgres; then
    echo "Starting PostgreSQL container..."
    docker-compose up -d
    echo "Waiting for PostgreSQL to be ready..."
    sleep 5
else
    echo "PostgreSQL is already running"
fi

# Функция для запуска сервиса в новом терминале
run_service() {
    local service_name=$1
    local port=$2
    
    echo -e "${GREEN}Starting $service_name on port $port...${NC}"
    
    # Для macOS
    if [[ "$OSTYPE" == "darwin"* ]]; then
        osascript -e "tell app \"Terminal\" to do script \"cd '$(pwd)' && cargo run -p $service_name\""
    else
        echo -e "${RED}Cannot detect terminal. Please run manually:${NC}"
        echo "cargo run -p $service_name"
    fi
}

echo -e "${YELLOW}Choose an option:${NC}"
echo "1) Start User Service only (port 50051)"
echo "2) Start Game Service only (port 50052)" 
echo "3) Start both services"
echo "4) Exit"

read -p "Enter your choice (1-4): " choice

case $choice in
    1)
        run_service "user-service" "50051"
        ;;
    2)
        run_service "game-service" "50052"
        ;;
    3)
        run_service "user-service" "50051"
        sleep 2
        run_service "game-service" "50052"
        ;;
    4)
        echo "Exiting..."
        exit 0
        ;;
    *)
        echo -e "${RED}Invalid choice${NC}"
        exit 1
        ;;
esac

echo -e "${GREEN}Services are starting...${NC}"
echo -e "${YELLOW}User Service:${NC} localhost:50051"
echo -e "${YELLOW}Game Service:${NC} localhost:50052"