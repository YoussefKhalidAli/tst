from http.server import BaseHTTPRequestHandler, HTTPServer
import threading
import time


class Handler(BaseHTTPRequestHandler):
    def log_request_data(self):
        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length) if content_length > 0 else b""

        print("\n===== Incoming Request =====", flush=True)
        print(f"Method: {self.command}", flush=True)
        print(f"Path: {self.path}", flush=True)

        print("Headers:", flush=True)
        for key, value in self.headers.items():
            print(f"  {key}: {value}", flush=True)

        if body:
            print(f"Body: {body.decode(errors='ignore')}", flush=True)

        print("============================\n", flush=True)

    def do_GET(self):
        self.log_request_data()

        self.send_response(200)
        self.send_header("Content-type", "text/plain")
        self.end_headers()
        self.wfile.write(b"Hello from GitHub! Attempt1")

    def do_POST(self):
        self.log_request_data()

        self.send_response(200)
        self.end_headers()
        self.wfile.write(b"POST received")

    def do_PUT(self):
        self.log_request_data()

        self.send_response(200)
        self.end_headers()
        self.wfile.write(b"PUT received")

    def do_DELETE(self):
        self.log_request_data()

        self.send_response(200)
        self.end_headers()
        self.wfile.write(b"DELETE received")

    def do_PATCH(self):
        self.log_request_data()

        self.send_response(200)
        self.end_headers()
        self.wfile.write(b"PATCH received")

    def do_OPTIONS(self):
        self.log_request_data()

        self.send_response(200)
        self.end_headers()

    def do_HEAD(self):
        self.log_request_data()

        self.send_response(200)
        self.end_headers()


def log_message():
    while True:
        print("Server is still running...", flush=True)
        time.sleep(10)


if __name__ == "__main__":
    # Start logger thread
    threading.Thread(target=log_message, daemon=True).start()

    server = HTTPServer(("0.0.0.0", 7777), Handler)

    print("Server running on port 7777", flush=True)

    server.serve_forever()
