declare global {
  interface Window {
    ENV: {
      BACKEND_URL: string;
    };
  }
}

export {};
