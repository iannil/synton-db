/**
 * E2E Tests: Edge Operations
 * Scenario 2: Create edge → Verify graph traversal
 */

import { test, expect } from './fixtures';
import { randomTestName } from './helpers';

test.describe('Edge Operations', () => {
  test('should create an edge between two nodes', async ({ api }) => {
    // Arrange
    const sourceNode = await api.createNode(randomTestName('Source: Paris'), 'Entity');
    const targetNode = await api.createNode(randomTestName('Target: France'), 'Entity');

    // Act
    const edge = await api.createEdge(
      sourceNode.id,
      targetNode.id,
      'is_part_of',
      0.9
    );

    // Assert
    expect(edge).toBeDefined();
    expect(edge.source).toBe(sourceNode.id);
    expect(edge.target).toBe(targetNode.id);
    expect(edge.relation).toBe('is_part_of');
    expect(edge.weight).toBe(0.9);
  });

  test('should support different relation types', async ({ api }) => {
    // Arrange
    const node1 = await api.createNode(randomTestName('Node 1'), 'Entity');
    const node2 = await api.createNode(randomTestName('Node 2'), 'Entity');

    // Act & Assert
    const isA = await api.createEdge(node1.id, node2.id, 'is_a', 1.0);
    expect(isA.relation).toBe('is_a');

    const isPartOf = await api.createEdge(node1.id, node2.id, 'is_part_of', 1.0);
    expect(isPartOf.relation).toBe('is_part_of');

    const causes = await api.createEdge(node1.id, node2.id, 'causes', 0.8);
    expect(causes.relation).toBe('causes');

    const similarTo = await api.createEdge(node1.id, node2.id, 'similar_to', 0.7);
    expect(similarTo.relation).toBe('similar_to');

    const contradicts = await api.createEdge(node1.id, node2.id, 'contradicts', 0.9);
    expect(contradicts.relation).toBe('contradicts');

    const happenedAfter = await api.createEdge(node1.id, node2.id, 'happened_after', 1.0);
    expect(happenedAfter.relation).toBe('happened_after');

    const belongsTo = await api.createEdge(node1.id, node2.id, 'belongs_to', 1.0);
    expect(belongsTo.relation).toBe('belongs_to');
  });

  test('should create a knowledge graph structure', async ({ api }) => {
    // Arrange - Create a small knowledge graph
    const paris = await api.createNode(randomTestName('Paris'), 'Entity');
    const france = await api.createNode(randomTestName('France'), 'Entity');
    const europe = await api.createNode(randomTestName('Europe'), 'Entity');
    const capital = await api.createNode(randomTestName('capital city'), 'Concept');

    // Act - Create edges
    await api.createEdge(paris.id, france.id, 'is_part_of', 1.0);
    await api.createEdge(france.id, europe.id, 'is_part_of', 1.0);
    await api.createEdge(paris.id, capital.id, 'is_a', 1.0);

    // Assert - Verify nodes exist
    const nodes = await api.listNodes();
    expect(nodes.length).toBeGreaterThanOrEqual(4);

    // Verify stats updated
    const stats = await api.stats();
    expect(stats.edge_count).toBeGreaterThanOrEqual(3);
  });

  test('should support relation aliases with hyphens', async ({ api }) => {
    // Arrange
    const node1 = await api.createNode(randomTestName('Node 1'), 'Entity');
    const node2 = await api.createNode(randomTestName('Node 2'), 'Entity');

    // Act - Using hyphenated form (API should convert to underscore)
    const edge = await api.createEdge(node1.id, node2.id, 'is-part-of', 1.0);

    // Assert
    expect(edge.relation).toBe('is_part_of');
  });
});
