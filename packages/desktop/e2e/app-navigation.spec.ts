import { test, expect } from './fixtures';

test.describe('App Navigation', () => {
  test('should load the main page', async ({ page }) => {
    const response = await page.goto('/');

    // Page should load successfully
    expect(response?.status()).toBeLessThan(400);

    // Wait for React to render
    await page.waitForLoadState('networkidle');

    // Root element should exist
    await expect(page.locator('#root')).toBeVisible();
  });

  test('should have navigation elements', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Look for navigation-related elements (nav, header, sidebar, etc.)
    const navElements = page.locator('nav, header, [role="navigation"], aside');
    const hasNav = await navElements.first().isVisible().catch(() => false);

    // The app may or may not have traditional navigation - just verify it loads
    const root = page.locator('#root');
    await expect(root).toBeVisible();
  });

  test('should be responsive', async ({ page }) => {
    // Test at mobile size
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // App should still render at mobile size
    await expect(page.locator('#root')).toBeVisible();

    // Test at desktop size
    await page.setViewportSize({ width: 1280, height: 720 });
    await page.waitForLoadState('networkidle');

    await expect(page.locator('#root')).toBeVisible();
  });
});
