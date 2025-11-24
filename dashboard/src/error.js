export class FileReadError extends Error {
  constructor(fileName, cause) {
    super(`Unable to read file "${fileName}": ${cause?.message ?? 'Unknown error'}`);
    this.name = 'FileReadError';
    this.fileName = fileName;
    this.cause = cause;
  }
}

export class DuckDBError extends Error {
  constructor(operation, cause) {
    super(`DuckDB operation failed (${operation}): ${cause?.message ?? 'Unknown error'}`);
    this.name = 'DuckDBError';
    this.operation = operation;
    this.cause = cause;
  }
}

export class ArrowParseError extends Error {
  constructor(fileName, cause) {
    super(`Failed to parse Arrow IPC data for "${fileName}": ${cause?.message ?? 'Unknown error'}`);
    this.name = 'ArrowParseError';
    this.fileName = fileName;
    this.cause = cause;
  }
}
