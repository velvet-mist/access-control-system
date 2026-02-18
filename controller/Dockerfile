FROM python:3.12-slim

WORKDIR /app

COPY controller/ ./controller

COPY controller/requirements.txt ./requirements.txt
RUN pip install --no-cache-dir -r requirements.txt

ENV PYTHONPATH=/app

CMD ["uvicorn", "controller.main:app", "--host", "0.0.0.0", "--port", "8000"]
