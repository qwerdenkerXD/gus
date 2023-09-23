import React from 'react';
import { render, screen } from '@testing-library/react';
import App from './App';

test('renders my github link', () => {
  render(<App />);
  const linkElement = screen.getByText(/my github/i);
  expect(linkElement).toBeInTheDocument();
});
