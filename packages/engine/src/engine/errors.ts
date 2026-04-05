/** Error type thrown by the engine runtime. */
export class EngineError extends Error {
  code: string;

  constructor(code: string, message: string) {
    super(message);
    this.code = code;
    this.name = 'EngineError';
  }
}
