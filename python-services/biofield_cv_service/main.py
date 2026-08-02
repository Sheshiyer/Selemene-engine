"""Biofield CV sidecar service.

FastAPI application. Listens on $PORT (default 8002) so Railway can route correctly.
"""

import os
import uvicorn
from fastapi import FastAPI
from shared.version import SERVICE_VERSION
from fastapi.middleware.cors import CORSMiddleware

from biofield_cv_service.health import router as health_router
from biofield_cv_service.analyze import router as analyze_router

app = FastAPI(
    title="Selemene Biofield CV Service",
    version=SERVICE_VERSION,
    description="Python sidecar service for biofield image analysis using OpenCV.",
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(health_router)
app.include_router(analyze_router)


def start() -> None:
    """Entry point for the `biofield-cv-service` console script."""
    port = int(os.environ.get("PORT", 8002))
    uvicorn.run(
        "biofield_cv_service.main:app",
        host="0.0.0.0",
        port=port,
        reload=False,
    )


if __name__ == "__main__":
    start()
