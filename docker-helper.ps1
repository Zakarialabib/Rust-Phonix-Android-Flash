# Phoenix Tools - Docker Helper for Windows
# Usage: .\docker-helper.ps1 <command>

param(
    [Parameter(Position=0)]
    [string]$Command = "help",
    
    [Parameter(ValueFromRemainingArguments=$true)]
    [string[]]$Args
)

$ErrorActionPreference = "Stop"

function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

function Show-Help {
    Write-ColorOutput "`n[Docker] Phoenix Tools - Docker Commands" "Cyan"
    Write-ColorOutput "=" "DarkGray"
    Write-Host ""
    Write-ColorOutput "Development:" "Green"
    Write-Host "  .\docker-helper.ps1 dev          - Start full development environment"
    Write-Host "  .\docker-helper.ps1 dev-cli      - Start CLI-only development"
    Write-Host "  .\docker-helper.ps1 dev-gui      - Start GUI-only development"
    Write-Host "  .\docker-helper.ps1 shell        - Open bash shell in dev container"
    Write-Host "  .\docker-helper.ps1 logs         - Follow container logs"
    Write-Host ""
    Write-ColorOutput "Building:" "Green"
    Write-Host "  .\docker-helper.ps1 build        - Build all images"
    Write-Host "  .\docker-helper.ps1 build-dev    - Build development image"
    Write-Host "  .\docker-helper.ps1 build-prod   - Build production image"
    Write-Host ""
    Write-ColorOutput "Production:" "Green"
    Write-Host "  .\docker-helper.ps1 prod         - Run production container"
    Write-Host "  .\docker-helper.ps1 cli <args>   - Run CLI command"
    Write-Host ""
    Write-ColorOutput "Testing:" "Green"
    Write-Host "  .\docker-helper.ps1 test         - Run all tests"
    Write-Host "  .\docker-helper.ps1 fmt          - Format code"
    Write-Host "  .\docker-helper.ps1 lint         - Run linter"
    Write-Host ""
    Write-ColorOutput "Maintenance:" "Green"
    Write-Host "  .\docker-helper.ps1 clean        - Remove containers and volumes"
    Write-Host "  .\docker-helper.ps1 clean-all    - Remove everything (CAUTION)"
    Write-Host "  .\docker-helper.ps1 up           - Start containers in background"
    Write-Host "  .\docker-helper.ps1 down         - Stop all containers"
    Write-Host "  .\docker-helper.ps1 restart      - Restart containers"
    Write-Host ""
    Write-ColorOutput "Examples:" "Yellow"
    Write-Host "  .\docker-helper.ps1 cli detect"
    Write-Host "  .\docker-helper.ps1 cli vault list"
    Write-Host "  .\docker-helper.ps1 cli forensics deep-scan --device /dev/ttyUSB0"
    Write-Host ""
}

function Test-DockerRunning {
    try {
        docker ps | Out-Null
        return $true
    } catch {
        Write-ColorOutput "[Error] Docker is not running. Please start Docker Desktop." "Red"
        return $false
    }
}

function Invoke-DockerCompose {
    param([string[]]$Arguments)
    
    if (-not (Test-DockerRunning)) {
        exit 1
    }
    
    & docker-compose $Arguments
}

# Main command router
switch ($Command.ToLower()) {
    "help" {
        Show-Help
    }
    
    "dev" {
        Write-ColorOutput "[Start] Starting development environment..." "Green"
        Invoke-DockerCompose @("up", "dev")
    }
    
    "dev-cli" {
        Write-ColorOutput "[Start] Starting CLI development..." "Green"
        Invoke-DockerCompose @("up", "dev-cli")
    }
    
    "dev-gui" {
        Write-ColorOutput "[Start] Starting GUI development..." "Green"
        Invoke-DockerCompose @("up", "dev-gui")
    }
    
    "shell" {
        Write-ColorOutput "[Shell] Opening development shell..." "Green"
        try {
            Invoke-DockerCompose @("exec", "dev", "bash")
        } catch {
            Invoke-DockerCompose @("run", "--rm", "dev", "bash")
        }
    }
    
    "logs" {
        Invoke-DockerCompose @("logs", "-f", "dev")
    }
    
    "build" {
        Write-ColorOutput "[Build] Building all images..." "Green"
        Invoke-DockerCompose @("build")
    }
    
    "build-dev" {
        Write-ColorOutput "[Build] Building development image..." "Green"
        Invoke-DockerCompose @("build", "dev")
    }
    
    "build-prod" {
        Write-ColorOutput "[Build] Building production image..." "Green"
        Invoke-DockerCompose @("build", "prod")
    }
    
    "prod" {
        Write-ColorOutput "[Start] Starting production container..." "Green"
        Invoke-DockerCompose @("up", "prod")
    }
    
    "cli" {
        if ($Args.Count -eq 0) {
            Write-ColorOutput "[Error] Please specify CLI arguments. Example: cli detect" "Red"
            exit 1
        }
        Write-ColorOutput "[Run] Running CLI command: $Args" "Green"
        Invoke-DockerCompose (@("run", "--rm", "cli") + $Args)
    }
    
    "test" {
        Write-ColorOutput "[Test] Running tests..." "Green"
        Invoke-DockerCompose @("run", "--rm", "test")
    }
    
    "fmt" {
        Write-ColorOutput "[Fmt] Formatting code..." "Green"
        Invoke-DockerCompose @("run", "--rm", "dev", "cargo", "fmt", "--all")
    }
    
    "lint" {
        Write-ColorOutput "[Lint] Running clippy..." "Green"
        Invoke-DockerCompose @("run", "--rm", "dev", "cargo", "clippy", "--workspace", "--all-features", "--", "-D", "warnings")
    }
    
    "up" {
        Write-ColorOutput "[Up] Starting containers..." "Green"
        Invoke-DockerCompose @("up", "-d")
    }
    
    "down" {
        Write-ColorOutput "[Down] Stopping containers..." "Yellow"
        Invoke-DockerCompose @("down")
    }
    
    "restart" {
        Write-ColorOutput "[Restart] Restarting containers..." "Yellow"
        Invoke-DockerCompose @("restart")
    }
    
    "clean" {
        Write-ColorOutput "[Clean] Cleaning up containers and volumes..." "Yellow"
        $confirm = Read-Host "Are you sure? (y/N)"
        if ($confirm -eq "y" -or $confirm -eq "Y") {
            Invoke-DockerCompose @("down", "-v")
            Write-ColorOutput "[Done] Cleanup complete!" "Green"
        } else {
            Write-ColorOutput "[Cancel] Cancelled" "Red"
        }
    }
    
    "clean-all" {
        Write-ColorOutput "[Warn] WARNING: This will remove all containers, volumes, and images!" "Red"
        $confirm = Read-Host "Are you ABSOLUTELY sure? (y/N)"
        if ($confirm -eq "y" -or $confirm -eq "Y") {
            Invoke-DockerCompose @("down", "-v", "--rmi", "all")
            Write-ColorOutput "[Done] Full cleanup complete!" "Green"
        } else {
            Write-ColorOutput "[Cancel] Cancelled" "Red"
        }
    }
    
    default {
        Write-ColorOutput "[Error] Unknown command: $Command" "Red"
        Write-Host ""
        Show-Help
        exit 1
    }
}
