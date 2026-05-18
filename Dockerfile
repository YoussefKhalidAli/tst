FROM python:3.11-slim

ENV PYTHONUNBUFFERED=1

WORKDIR /app

COPY app.py .

EXPOSE 8000

CMD ["python", "app.py"]
