import {NavDropdown as BNavDropdown} from "react-bootstrap";

function NavbarAdvance() {
    return (
        <BNavDropdown title="Advance" id="navbar-advance">
            <BNavDropdown.Item href="#advance-logs">Show logs</BNavDropdown.Item>
            <BNavDropdown.Item href="#advance-blockchain">Show Blockchain</BNavDropdown.Item>
        </BNavDropdown>
    )
}

export default NavbarAdvance;
