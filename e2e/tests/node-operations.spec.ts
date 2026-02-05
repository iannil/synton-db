/**
 * E2E Tests: Node Operations
 * Scenario 1: Create node → Verify storage
 */

import { test, expect } from './fixtures';
import { randomTestName } from './helpers';

test.describe('Node Operations', () => {
  test('should create a node and verify it is stored', async ({ api }) => {
    // Arrange
    const content = randomTestName('Test node content');
    const nodeType = 'Concept';

    // Act
    const createdNode = await api.createNode(content, nodeType);

    // Assert
    expect(createdNode).toBeDefined();
    expect(createdNode.id).toBeDefined();
    expect(createdNode.content).toBe(content);
    // API returns lowercase node types
    expect(createdNode.node_type).toBe('concept');
    expect(createdNode.meta.access_score).toBe(1.0);
    expect(createdNode.meta.confidence).toBe(1.0);
  });

  test('should retrieve a created node by ID', async ({ api }) => {
    // Arrange
    const content = randomTestName('Retrievable node');
    const createdNode = await api.createNode(content, 'Concept');

    // Act
    const retrievedNode = await api.getNode(createdNode.id);

    // Assert
    expect(retrievedNode).not.toBeNull();
    expect(retrievedNode?.id).toBe(createdNode.id);
    expect(retrievedNode?.content).toBe(content);
  });

  test('should return null for non-existent node', async ({ api }) => {
    // Act
    const node = await api.getNode('00000000-0000-0000-0000-000000000000');

    // Assert
    expect(node).toBeNull();
  });

  test('should list all created nodes', async ({ api }) => {
    // Arrange
    await api.createNode(randomTestName('Node 1'), 'Concept');
    await api.createNode(randomTestName('Node 2'), 'Entity');
    await api.createNode(randomTestName('Node 3'), 'Fact');

    // Act
    const nodes = await api.listNodes();

    // Assert
    expect(nodes.length).toBeGreaterThanOrEqual(3);
  });

  test('should delete a node', async ({ api }) => {
    // Arrange
    const createdNode = await api.createNode(randomTestName('Deletable node'), 'Concept');

    // Act
    const deleted = await api.deleteNode(createdNode.id);

    // Assert
    expect(deleted).toBe(true);
    const retrieved = await api.getNode(createdNode.id);
    expect(retrieved).toBeNull();
  });

  test('should support different node types', async ({ api }) => {
    // Act & Assert
    const entity = await api.createNode(randomTestName('Entity: Paris'), 'Entity');
    expect(entity.node_type).toBe('entity');

    const concept = await api.createNode(randomTestName('Concept: Democracy'), 'Concept');
    expect(concept.node_type).toBe('concept');

    const fact = await api.createNode(randomTestName('Fact: E=mc²'), 'Fact');
    expect(fact.node_type).toBe('fact');

    const chunk = await api.createNode(randomTestName('Chunk: text fragment'), 'RawChunk');
    expect(chunk.node_type).toBe('raw_chunk');
  });
});
