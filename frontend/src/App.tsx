import React from 'react';
import Navbar from './Navbar';
import './App.css';
import { Routes, Route } from 'react-router-dom';
import Home from './Home';

function App() {
  return (
      <>
          <div className="App">
              <Navbar />
          </div>
          <Routes>
              <Route path="/home" element={<Home />} />
          </Routes>
      </>
  );
}

export default App;
