/**
 * E2E Tests: API Response Format Validation
 * Validates API response structures and error handling
 */

import { test, expect } from './fixtures';
import { randomTestName } from './helpers';

test.describe('API Response Format', () => {
  test('should return correct node structure', async ({ api }) => {
    // Act
    const node = await api.createNode('Test content', 'Concept');

    // Assert - validate node structure
    expect(node).toHaveProperty('id');
    expect(node).toHaveProperty('content');
    expect(node).toHaveProperty('node_type');
    expect(node).toHaveProperty('meta');
    expect(node).toHaveProperty('attributes');
    expect(node.meta).toHaveProperty('created_at');
    expect(node.meta).toHaveProperty('updated_at');
    expect(node.meta).toHaveProperty('access_score');
    expect(node.meta).toHaveProperty('confidence');
    expect(node.meta).toHaveProperty('source');
  });

  test('should return correct edge structure', async ({ api }) => {
    // Arrange
    const node1 = await api.createNode(randomTestName('Source'), 'Entity');
    const node2 = await api.createNode(randomTestName('Target'), 'Entity');

    // Act
    const edge = await api.createEdge(node1.id, node2.id, 'is_a', 0.9);

    // Assert - validate edge structure
    expect(edge).toHaveProperty('source');
    expect(edge).toHaveProperty('target');
    expect(edge).toHaveProperty('relation');
    expect(edge).toHaveProperty('weight');
    expect(edge).toHaveProperty('created_at');
    expect(edge).toHaveProperty('vector');
    expect(edge).toHaveProperty('expired');
    expect(edge).toHaveProperty('replaced_by');
    expect(edge).toHaveProperty('attributes');
    expect(edge.source).toBe(node1.id);
    expect(edge.target).toBe(node2.id);
  });

  test('should return correct query response structure', async ({ api }) => {
    // Arrange
    await api.createNode('Test query response', 'Concept');

    // Act
    const response = await api.query('test');

    // Assert - validate query response structure
    expect(response).toHaveProperty('nodes');
    expect(response).toHaveProperty('total_count');
    expect(response).toHaveProperty('execution_time_ms');
    expect(response).toHaveProperty('truncated');
    expect(Array.isArray(response.nodes)).toBe(true);
  });

  test('should return correct stats response structure', async ({ api }) => {
    // Act
    const stats = await api.stats();

    // Assert - validate stats structure
    expect(stats).toHaveProperty('node_count');
    expect(stats).toHaveProperty('edge_count');
    expect(stats).toHaveProperty('embedded_count');
    expect(typeof stats.node_count).toBe('number');
    expect(typeof stats.edge_count).toBe('number');
    expect(typeof stats.embedded_count).toBe('number');
  });

  test('should return correct health response structure', async ({ api }) => {
    // Act
    const health = await api.health();

    // Assert - validate health structure
    expect(health).toHaveProperty('status');
    expect(health).toHaveProperty('version');
    expect(health.status).toBe('healthy');
  });
});

test.describe('API Error Handling', () => {
  test('should return 404 for non-existent node', async ({ api }) => {
    // Act
    const node = await api.getNode('00000000-0000-0000-0000-000000000000');

    // Assert
    expect(node).toBeNull();
  });

  test('should handle invalid UUID format gracefully', async ({ api }) => {
    // Act - trying to get with invalid UUID
    const response = await fetch(`${api['baseUrl']}/nodes/invalid-uuid`);

    // Assert - should return error status
    expect(response.status).toBeGreaterThanOrEqual(400);
  });

  test('should handle empty query gracefully', async ({ api }) => {
    // Act
    const response = await api.query('');

    // Assert - should return valid response structure
    expect(response).toHaveProperty('nodes');
    expect(response).toHaveProperty('total_count');
  });
});
