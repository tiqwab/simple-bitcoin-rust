import NavbarMenu from './NavbarMenu';
import NavbarSettings from './NavbarSettings';
import NavbarAdvance from './NavbarAdvance';
import {Container as BContainer, Nav as BNav, Navbar as BNavbar} from "react-bootstrap";

function Navbar() {
    return (
        <BNavbar bg="light" expand="lg">
            <BContainer>
                <BNavbar.Brand href="#home">Simple Bitcoin</BNavbar.Brand>
                <BNavbar.Toggle aria-controls="basic-navbar-nav"/>
                <BNavbar.Collapse id="basic-navbar-nav">
                    <BNav className="me-auto">
                        <BNav.Link href="#home">Home</BNav.Link>
                        <NavbarMenu/>
                        <NavbarSettings/>
                        <NavbarAdvance/>
                    </BNav>
                </BNavbar.Collapse>
            </BContainer>
        </BNavbar>
    )
}

export default Navbar;