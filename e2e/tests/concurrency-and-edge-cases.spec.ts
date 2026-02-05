/**
 * E2E Tests: Concurrency and Edge Cases
 * Tests concurrent operations and boundary conditions
 */

import { test, expect } from './fixtures';
import { randomTestName } from './helpers';

test.describe('Edge Cases', () => {
  test('should handle very long content', async ({ api }) => {
    // Arrange - create node with very long content (10000 chars)
    const longContent = 'A'.repeat(10000);

    // Act
    const node = await api.createNode(longContent, 'Concept');

    // Assert
    expect(node.content).toBe(longContent);
  });

  test('should handle special characters in content', async ({ api }) => {
    // Arrange
    const specialContent = 'Test with special chars: <>&"\'\\n\\t🚀';

    // Act
    const node = await api.createNode(specialContent, 'Concept');

    // Assert
    expect(node.content).toBe(specialContent);
  });

  test('should handle unicode characters', async ({ api }) => {
    // Arrange
    const unicodeContent = 'Hello 世界 🌍 مرحبا Привет';

    // Act
    const node = await api.createNode(unicodeContent, 'Concept');

    // Assert
    expect(node.content).toBe(unicodeContent);
  });

  test('should handle empty content', async ({ api }) => {
    // Act
    const node = await api.createNode('', 'Concept');

    // Assert
    expect(node.content).toBe('');
  });

  test('should handle edge weight boundaries', async ({ api }) => {
    // Arrange
    const node1 = await api.createNode(randomTestName('Node 1'), 'Entity');
    const node2 = await api.createNode(randomTestName('Node 2'), 'Entity');

    // Act - create edges with min and max weights
    const minEdge = await api.createEdge(node1.id, node2.id, 'is_a', 0.0);
    const maxEdge = await api.createEdge(node1.id, node2.id, 'is_part_of', 1.0);

    // Assert
    expect(minEdge.weight).toBe(0.0);
    expect(maxEdge.weight).toBe(1.0);
  });

  test('should handle same source and target nodes', async ({ api }) => {
    // Arrange
    const nodeId = (await api.createNode(randomTestName('Self node'), 'Entity')).id;

    // Act - create edge with same source and target
    const edge = await api.createEdge(nodeId, nodeId, 'similar_to', 1.0);

    // Assert - API may allow or reject self-referencing edges
    expect(edge).toBeDefined();
    expect(edge.source).toBe(nodeId);
    expect(edge.target).toBe(nodeId);
  });
});

test.describe('Batch Operations', () => {
  test('should handle creating multiple nodes in sequence', async ({ api }) => {
    // Arrange
    const nodeCount = 50;
    const nodeIds: string[] = [];

    // Act - create many nodes
    for (let i = 0; i < nodeCount; i++) {
      const node = await api.createNode(`${randomTestName(`Batch node ${i}`)}`, 'Concept');
      nodeIds.push(node.id);
    }

    // Assert - all nodes should be created
    expect(nodeIds.length).toBe(nodeCount);

    // Verify a few nodes
    const sampleNode = await api.getNode(nodeIds[0]);
    expect(sampleNode).not.toBeNull();
    expect(sampleNode?.id).toBe(nodeIds[0]);
  });

  test('should handle creating multiple edges in sequence', async ({ api }) => {
    // Arrange - create a chain of nodes
    const nodes: any[] = [];
    for (let i = 0; i < 10; i++) {
      nodes.push(await api.createNode(randomTestName(`Chain node ${i}`), 'Entity'));
    }

    // Act - create edges linking the chain
    for (let i = 0; i < nodes.length - 1; i++) {
      await api.createEdge(nodes[i].id, nodes[i + 1].id, 'is_part_of', 1.0);
    }

    // Assert - verify stats updated
    const stats = await api.stats();
    expect(stats.edge_count).toBeGreaterThanOrEqual(9);
  });
});

test.describe('Query Edge Cases', () => {
  test('should handle very long query string', async ({ api }) => {
    // Arrange
    const longQuery = 'a '.repeat(1000);

    // Act
    const response = await api.query(longQuery);

    // Assert - should return valid response
    expect(response).toHaveProperty('nodes');
    expect(response).toHaveProperty('total_count');
  });

  test('should handle query with no results', async ({ api }) => {
    // Act
    const response = await api.query('xyznonexistent12345');

    // Assert
    expect(response.total_count).toBe(0);
    expect(response.nodes).toHaveLength(0);
  });

  test('should handle case sensitivity in queries', async ({ api }) => {
    // Arrange
    await api.createNode('Paris is the capital of France', 'Fact');

    // Act - query with different cases
    const lowerResult = await api.query('paris');
    const upperResult = await api.query('PARIS');
    const mixedResult = await api.query('PaRiS');

    // Assert - at least one should find the node
    const results = [lowerResult, upperResult, mixedResult];
    const foundCount = results.filter(r => r.total_count > 0).length;
    expect(foundCount).toBeGreaterThan(0);
  });
});
