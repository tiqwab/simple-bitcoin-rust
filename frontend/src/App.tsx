import React from 'react';
import Navbar from './Navbar';
import './App.css';
import {Routes, Route} from 'react-router-dom';
import Home from './Home';
import UpdateBlockchain from "./UpdateBlockchain";
import {ToastProvider} from "./useToast";

function App() {
    return (
        <>
            <ToastProvider>
                <div className="App">
                    <Navbar/>
                </div>
                <Routes>
                    <Route path="/home" element={<Home/>}/>
                    <Route path="/update-blockchain"
                           element={<UpdateBlockchain/>}/>
                </Routes>
            </ToastProvider>
        </>
    );
}

export default App;
