"""Integration tests for Selemene Universal Tool Bridge.

Run with: pytest bridges/tests/test_bridge.py -v
Requires: Selemene server + TS engines + bridge running
"""

import pytest
import httpx
import os

BRIDGE_URL = os.environ.get("BRIDGE_URL", "http://localhost:8000")


@pytest.fixture
def client():
    return httpx.Client(base_url=BRIDGE_URL, timeout=30.0)


class TestBridgeHealth:
    def test_bridge_health(self, client):
        r = client.get("/health")
        assert r.status_code == 200
        data = r.json()
        assert data["bridge"] == "healthy"

    def test_bridge_lists_tools(self, client):
        r = client.get("/tools")
        assert r.status_code == 200
        data = r.json()
        assert data["count"] > 0
        names = [t["name"] for t in data["tools"]]
        assert "health" in names
        assert "engine_calculate" in names
        assert "workflow_execute" in names


class TestToolExecution:
    def test_health_tool(self, client):
        r = client.post("/tools/execute", json={"name": "health", "arguments": {}})
        assert r.status_code == 200
        data = r.json()
        assert data["success"] is True
        assert "data" in data

    def test_list_engines(self, client):
        r = client.post("/tools/execute", json={"name": "list_engines", "arguments": {}})
        assert r.status_code == 200
        data = r.json()
        assert data["tool"] == "list_engines"

    def test_engine_info(self, client):
        r = client.post("/tools/execute", json={
            "name": "engine_info",
            "arguments": {"engine_id": "panchanga"}
        })
        assert r.status_code == 200

    def test_list_workflows(self, client):
        r = client.post("/tools/execute", json={"name": "list_workflows", "arguments": {}})
        assert r.status_code == 200

    def test_workflow_info(self, client):
        r = client.post("/tools/execute", json={
            "name": "workflow_info",
            "arguments": {"workflow_id": "daily-practice"}
        })
        assert r.status_code == 200


class TestErrorHandling:
    def test_unknown_tool(self, client):
        r = client.post("/tools/execute", json={"name": "nonexistent", "arguments": {}})
        assert r.status_code == 404

    def test_invalid_engine_id(self, client):
        r = client.post("/tools/execute", json={
            "name": "engine_info",
            "arguments": {"engine_id": "nonexistent-engine"}
        })
        assert r.status_code == 200  # Bridge returns 200 with error in body
        data = r.json()
        assert data["success"] is False

    def test_missing_arguments(self, client):
        r = client.post("/tools/execute", json={"name": "engine_calculate", "arguments": {}})
        assert r.status_code == 200
        data = r.json()
        # Should fail because engine_id path param is missing
        assert data["success"] is False


class TestTsEngines:
    def test_ts_health(self, client):
        r = client.post("/tools/execute", json={"name": "ts_health", "arguments": {}})
        assert r.status_code == 200

    def test_ts_list_engines(self, client):
        r = client.post("/tools/execute", json={"name": "ts_list_engines", "arguments": {}})
        assert r.status_code == 200
