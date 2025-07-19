# Code Architecture & Data Flow Documentation

## Overview
This document provides a detailed technical overview of the `aer_parser` codebase, focusing on the `src/` directory structure, module responsibilities, and data flow patterns.

## Directory Structure
```mermaid
graph TD
    src[src/] --> main[main.rs]
    src --> lib[lib.rs]
    src --> st1[st1.rs]
    src --> st49[st49.rs]
    src --> delta[delta.rs]
    src --> downloader[downloader.rs]
    src --> utils[utils.rs]
    src --> error[error.rs]
    src --> parsers[parsers/]
    
    parsers --> common[common.rs]
    parsers --> error_parser[error.rs]
    parsers --> traits[traits.rs]
    parsers --> mod_parser[mod.rs]
```

## Module Deep Dive

### 1. main.rs - CLI Entry Point
**Responsibility**: Command-line interface orchestration and workflow coordination

**Data Flow**:
```mermaid
flowchart TD
    A[CLI Arguments] --> B[Command Parser]
    B --> C{Command Type}
    C --> D[File Processing]
    C --> E[Folder Processing]
    C --> F[Date Range Processing]
    C --> G[Zip Processing]
    C --> H[Delta Loading]
    
    D --> I[process_file]
    E --> J[process_folder]
    F --> K[process_date_range]
    G --> L[process_zip_folder]
    H --> M[load_delta_workflow]
    
    style A fill:#f9f,stroke:#333
    style M fill:#9f9,stroke:#333
```

**Key Functions**:
- `main()`: Entry point with async runtime
- Command routing based on CLI subcommands
- Delta Lake loading orchestration

### 2. lib.rs - Library Interface
**Responsibility**: Public API exports and shared types

**Exports**:
- `ReportType` enum (St1, St49)
- `AppError` type
- Processing functions: `process_file`, `process_folder`, `process_date_range`, `process_zip_folder`

### 3. st1.rs - ST1 Report Parser
**Responsibility**: Parsing ST1 (WELLS) reports

**Data Structure**:
```mermaid
classDiagram
    class ST1Record {
        +date: String
        +well_name: String
        +licence_number: String
        +mineral_rights: String
        +ground_elevation: String
        +unique_identifier: String
        +surface_coordinates: String
        +aer_field_centre: String
        +projected_depth: String
        +aer_classification: String
        +field: String
        +terminating_zone: String
        +drilling_operation: String
        +well_purpose: String
        +well_type: String
        +substance: String
        +licensee: String
        +surface_location: String
    }
```

**Parsing Flow**:
```mermaid
flowchart LR
    A[Raw ST1 Text] --> B[Line Parser]
    B --> C[Field Extractor]
    C --> D[Validation]
    D --> E[ST1Record]
    E --> F[CSV Writer]
    
    style A fill:#f9f,stroke:#333
    style F fill:#9f9,stroke:#333
```

### 4. st49.rs - ST49 Report Parser
**Responsibility**: Parsing ST49 (SPUD) reports

**Data Structure**:
```mermaid
classDiagram
    class ST49Record {
        +date: String
        +well_id: String
        +well_name: String
        +licence: String
        +contractor_ba_id: String
        +contractor_name: String
        +rig_number: String
        +activity_date: String
        +field_centre: String
        +ba_id: String
        +licensee: String
        +new_projected_total_depth: String
        +activity_type: String
    }
```

**Parsing Flow**:
```mermaid
flowchart LR
    A[Raw ST49 Text] --> B[Line Parser]
    B --> C[Field Extractor]
    C --> D[Validation]
    D --> E[ST49Record]
    E --> F[CSV Writer]
    
    style A fill:#f9f,stroke:#333
    style F fill:#9f9,stroke:#333
```

### 5. delta.rs - Delta Lake Integration
**Responsibility**: Delta Lake table creation, data loading, and maintenance

**Architecture**:
```mermaid
graph TD
    subgraph "Delta Operations"
        A[DeltaOps] --> B[Create Table]
        A --> C[Load Data]
        A --> D[Optimize]
        A --> E[Vacuum]
    end
    
    subgraph "Schema Management"
        F[Schema Builder] --> G[ST1 Schema]
        F --> H[ST49 Schema]
    end
    
    subgraph "Logging"
        I[Log Manager] --> J[Track Processed Files]
        I --> K[Error Handling]
    end
```

**Key Components**:
- `create_or_open_delta_table()`: Table initialization
- `load_csv_to_delta()`: Data ingestion
- `read_load_log()`: Process tracking
- `log_loaded_csv()`: Audit trail

### 6. downloader.rs - File Downloading
**Responsibility**: Asynchronous file retrieval from AER

**Download Flow**:
```mermaid
sequenceDiagram
    participant Main as Main Module
    participant Downloader as Downloader
    participant AER as AER Server
    participant Storage as Local Storage
    
    Main->>Downloader: Request date range
    Downloader->>AER: HTTP GET request
    AER-->>Downloader: File data
    Downloader->>Storage: Save to TXT directory
    Downloader-->>Main: Completion status
```

### 7. utils.rs - Utility Functions
**Responsibility**: Shared utilities and helper functions

**Utilities**:
- Date parsing and formatting
- File path handling
- String manipulation
- Error context creation

### 8. error.rs - Error Handling
**Responsibility**: Centralized error types and handling

**Error Hierarchy**:
```mermaid
graph TD
    AppError[AppError]
    AppError --> IoError[IoError]
    AppError --> ParseError[ParseError]
    AppError --> ValidationError[ValidationError]
    AppError --> DeltaError[DeltaError]
    AppError --> DownloadError[DownloadError]
```

### 9. parsers/ - Parser Utilities
**Responsibility**: Shared parsing infrastructure

#### parsers/common.rs
- File reading utilities
- CSV writing with pipe delimiter
- Progress reporting

#### parsers/error.rs
- Parser-specific error types
- Context creation for parsing failures

#### parsers/traits.rs
- Trait definitions for parser interfaces
- Shared behavior contracts

#### parsers/mod.rs
- Module exports and organization

## Data Flow Patterns

### 1. File Processing Pipeline
```mermaid
flowchart TD
    subgraph "Input"
        A[Raw TXT File]
    end
    
    subgraph "Processing"
        B[File Reader]
        C[Line Parser]
        D[Field Extractor]
        E[Record Validator]
    end
    
    subgraph "Output"
        F[CSV File]
        G[Delta Lake Table]
    end
    
    A --> B
    B --> C
    C --> D
    D --> E
    E --> F
    F --> G
```

### 2. Delta Lake Loading Pipeline
```mermaid
flowchart TD
    subgraph "Discovery"
        A[CSV Directory]
        B[File Filter]
        C[Log Check]
    end
    
    subgraph "Processing"
        D[CSV Reader]
        E[Record Batch]
        F[Delta Writer]
    end
    
    subgraph "Optimization"
        G[Table Optimize]
        H[Table Vacuum]
    end
    
    A --> B
    B --> C
    C --> D
    D --> E
    E --> F
    F --> G
    G --> H
```

### 3. Error Handling Flow
```mermaid
flowchart TD
    A[Operation Start]
    B{Error Occurred?}
    C[Log Error Context]
    D[Recovery Action]
    E[Continue Processing]
    F[Report to User]
    
    A --> B
    B -->|Yes| C
    C --> D
    D --> E
    B -->|No| E
    E --> F
```

## Memory Management

### Streaming Operations
```mermaid
graph TD
    subgraph "Memory Efficient Processing"
        A[Large File] --> B[Stream Reader]
        B --> C[Line Buffer]
        C --> D[Process Line]
        D --> E[Write CSV]
        E --> F[Flush Buffer]
        
        style A fill:#f9f,stroke:#333
        style F fill:#9f9,stroke:#333
    end
```

### Delta Lake Memory Usage
```mermaid
graph TD
    subgraph "Delta Operations"
        A[CSV Files] --> B[Batch Processing]
        B --> C[Memory Buffer]
        C --> D[Delta Write]
        D --> E[Commit Transaction]
        E --> F[Release Memory]
        
        style A fill:#f9f,stroke:#333
        style F fill:#9f9,stroke:#333
    end
```

## Configuration Patterns

### CLI Argument Processing
```mermaid
flowchart TD
    A[CLI Args] --> B[Clap Parser]
    B --> C[Validation]
    C --> D[Type Conversion]
    D --> E[Configuration]
    E --> F[Processing Pipeline]
```

### Environment Configuration
```mermaid
graph TD
    subgraph "Configuration Sources"
        A[CLI Args]
        B[Environment Variables]
        C[Default Values]
    end
    
    subgraph "Configuration Merge"
        D[Config Builder]
        E[Validation]
        F[Final Config]
    end
    
    A --> D
    B --> D
    C --> D
    D --> E
    E --> F
```

## Testing Architecture

### Unit Test Structure
```mermaid
graph TD
    subgraph "Test Organization"
        A[Unit Tests]
        B[Integration Tests]
        C[Benchmark Tests]
    end
    
    subgraph "Test Coverage"
        A --> D[Parser Logic]
        A --> E[Error Handling]
        B --> F[End-to-End]
        C --> G[Performance]
    end
```

### Test Data Flow
```mermaid
sequenceDiagram
    participant Test as Test Suite
    participant Fixture as Test Fixtures
    participant Parser as Parser Under Test
    participant Validator as Result Validator
    
    Test->>Fixture: Load test data
    Test->>Parser: Execute parsing
    Parser-->>Validator: Return results
    Validator-->>Test: Assert correctness
```

## Performance Considerations

### Memory Optimization
- **Streaming**: Large files processed line-by-line
- **Buffering**: Controlled memory usage with configurable buffer sizes
- **Lazy Loading**: Data loaded only when needed

### CPU Optimization
- **Parallel Processing**: Async operations for I/O bound tasks
- **Efficient Algorithms**: Optimized parsing for field extraction
- **Minimal Allocations**: Reuse of string buffers where possible

### Storage Optimization
- **Delta Lake**: Automatic file compaction and cleanup
- **Compression**: Zstd compression for parquet files
- **Partitioning**: Date-based partitioning for efficient queries

## Extension Points

### Adding New Report Types
1. Create new parser module (e.g., `st2.rs`)
2. Implement parsing logic following ST1/ST49 patterns
3. Add to CLI commands
4. Update Delta Lake schema

### Custom Output Formats
1. Implement new writer in `parsers/common.rs`
2. Add CLI option for format selection
3. Update configuration handling

### Enhanced Error Recovery
1. Extend error types in `error.rs`
2. Add recovery strategies in parsers
3. Update logging and reporting
